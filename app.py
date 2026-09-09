import os
import io
import json
import time
import uuid
import asyncio
import tempfile
import threading
from typing import Optional

from fastapi import FastAPI, UploadFile, File, Form, HTTPException
from fastapi.responses import StreamingResponse, JSONResponse
from reportlab.lib.pagesizes import A4
from reportlab.lib import colors
from reportlab.lib.styles import getSampleStyleSheet
from reportlab.platypus import SimpleDocTemplate, Table, TableStyle, Paragraph, Spacer

import spelling_checker as sc

app = FastAPI(title="Vietnamese Spelling Checker")

MAX_UPLOAD_BYTES = 200 * 1024 * 1024
CHUNK_SIZE = 1024 * 1024
SPREADSHEET_EXTS = (".xlsx", ".xlsm")

JOBS: dict = {}
JOBS_LOCK = threading.Lock()


def extract_units(path: str, ext: str, sheet_name: Optional[str]):
    units = []
    if ext in SPREADSHEET_EXTS:
        raw = sc.read_sheet(path, sheet_name or "Sheet1")
        for entry in json.loads(raw):
            location = f"R{entry['row'] + 1}C{entry['col'] + 1}"
            value = str(entry.get("value", ""))
            if value.strip():
                units.append((location, value))
    else:
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            for i, line in enumerate(f):
                line = line.rstrip("\n")
                if line.strip():
                    units.append((f"L{i + 1}", line))
    return units


def check_unit(location: str, text: str, whitelist: list, lang: str):
    errors = []
    for token in sc.tokenize(text):
        for sub, tag in sc.tag(token):
            if tag != "WORD":
                continue
            if sc.check(sub, whitelist, lang):
                errors.append({"location": location, "token": sub})
    return errors


def process_job(job_id: str, units: list, whitelist: list, lang: str):
    total = len(units)
    if total == 0:
        with JOBS_LOCK:
            JOBS[job_id]["status"] = "done"
            JOBS[job_id]["finished_at"] = time.time()
        yield 0, 0
        return

    step = max(1, total // 100)
    processed = 0
    errors = []
    for location, text in units:
        errors.extend(check_unit(location, text, whitelist, lang))
        processed += 1
        if processed % step == 0 or processed == total:
            with JOBS_LOCK:
                JOBS[job_id]["processed"] = processed
                JOBS[job_id]["progress"] = processed / total
            yield processed, total

    with JOBS_LOCK:
        JOBS[job_id]["status"] = "done"
        JOBS[job_id]["errors"] = errors
        JOBS[job_id]["finished_at"] = time.time()


async def save_upload(upload: UploadFile):
    suffix = os.path.splitext(upload.filename or "")[1].lower()
    fd, path = tempfile.mkstemp(suffix=suffix)
    size = 0
    try:
        with os.fdopen(fd, "wb") as out:
            while True:
                chunk = await upload.read(CHUNK_SIZE)
                if not chunk:
                    break
                size += len(chunk)
                if size > MAX_UPLOAD_BYTES:
                    raise HTTPException(status_code=413, detail="file too large")
                out.write(chunk)
    except HTTPException:
        os.remove(path)
        raise
    except Exception as e:
        os.remove(path)
        raise HTTPException(status_code=400, detail=str(e))
    return path, suffix


def build_pdf(job: dict) -> io.BytesIO:
    buffer = io.BytesIO()
    doc = SimpleDocTemplate(buffer, pagesize=A4)
    styles = getSampleStyleSheet()
    elements = [Paragraph("Báo cáo lỗi chính tả", styles["Title"]), Spacer(1, 12)]

    data = [["Vị trí", "Từ"]]
    for err in job["errors"]:
        data.append([err["location"], err["token"]])

    table = Table(data, colWidths=[100, 380])
    table.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), colors.HexColor("#2d3748")),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.white),
                ("GRID", (0, 0), (-1, -1), 0.5, colors.grey),
                ("FONTSIZE", (0, 0), (-1, -1), 9),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
            ]
        )
    )
    elements.append(table)
    doc.build(elements)
    buffer.seek(0)
    return buffer


@app.post("/check/stream")
async def check_stream(
    file: UploadFile = File(...),
    lang: str = Form("vi"),
    sheet_name: Optional[str] = Form(None),
    whitelist: Optional[str] = Form(None),
):
    path, ext = await save_upload(file)
    wl = [w.strip() for w in whitelist.split(",")] if whitelist else []

    try:
        units = await asyncio.to_thread(extract_units, path, ext, sheet_name)
    except Exception as e:
        os.remove(path)
        raise HTTPException(status_code=400, detail=str(e))

    job_id = uuid.uuid4().hex
    with JOBS_LOCK:
        JOBS[job_id] = {
            "status": "running",
            "processed": 0,
            "total": len(units),
            "progress": 0.0,
            "errors": [],
            "created_at": time.time(),
        }

    async def event_source():
        try:
            gen = process_job(job_id, units, wl, lang)
            while True:
                try:
                    processed, total = await asyncio.to_thread(next, gen)
                except StopIteration:
                    break
                payload = {
                    "job_id": job_id,
                    "processed": processed,
                    "total": total,
                    "progress": (processed / total) if total else 1.0,
                }
                yield f"data: {json.dumps(payload)}\n\n"
            with JOBS_LOCK:
                final = JOBS[job_id]
            done_payload = {
                "job_id": job_id,
                "status": "done",
                "total": final["total"],
                "errors_found": len(final["errors"]),
            }
            yield f"data: {json.dumps(done_payload)}\n\n"
        finally:
            os.remove(path)

    return StreamingResponse(event_source(), media_type="text/event-stream")


@app.post("/check/text")
async def check_text(
    text: str = Form(...),
    lang: str = Form("vi"),
    whitelist: Optional[str] = Form(None),
):
    wl = [w.strip() for w in whitelist.split(",")] if whitelist else []
    errors = await asyncio.to_thread(check_unit, "text", text, wl, lang)
    return JSONResponse({"errors": errors})


@app.get("/check/{job_id}")
async def get_job(job_id: str):
    with JOBS_LOCK:
        job = JOBS.get(job_id)
    if job is None:
        raise HTTPException(status_code=404, detail="job not found")
    return JSONResponse(
        {
            "status": job["status"],
            "processed": job["processed"],
            "total": job["total"],
            "progress": job["progress"],
            "errors": job["errors"],
        }
    )


@app.get("/check/{job_id}/pdf")
async def get_job_pdf(job_id: str):
    with JOBS_LOCK:
        job = JOBS.get(job_id)
    if job is None:
        raise HTTPException(status_code=404, detail="job not found")
    if job["status"] != "done":
        raise HTTPException(status_code=409, detail="job not finished")

    buffer = await asyncio.to_thread(build_pdf, job)
    headers = {"Content-Disposition": f'attachment; filename="report_{job_id}.pdf"'}
    return StreamingResponse(buffer, media_type="application/pdf", headers=headers)


@app.delete("/check/{job_id}")
async def delete_job(job_id: str):
    with JOBS_LOCK:
        existed = JOBS.pop(job_id, None) is not None
    if not existed:
        raise HTTPException(status_code=404, detail="job not found")
    return {"deleted": job_id}


@app.get("/health")
async def health():
    return {"status": "ok"}
