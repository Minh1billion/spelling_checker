# syntax=docker/dockerfile:1.7

FROM rust:1-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config patchelf libhunspell-dev python3 python3-dev python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --break-system-packages maturin

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    maturin build --release -o /wheels

FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-pip libhunspell-1.7-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /wheels/*.whl /tmp/
COPY requirements.txt .
RUN pip install --break-system-packages --no-cache-dir /tmp/*.whl -r requirements.txt

COPY dictionaries ./dictionaries
COPY app.py .

EXPOSE 8501
CMD ["streamlit", "run", "app.py", "--server.address=0.0.0.0", "--server.port=8501"]