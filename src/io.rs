use calamine::{Data, Reader, Xlsx};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufReader, Cursor};

pub fn read_sheet(source: &str, sheet_name: &str) -> PyResult<String> {
    let range = if source.starts_with("http://") || source.starts_with("https://") {
        let bytes = ureq::get(source)
            .call()
            .map_err(|e| PyIOError::new_err(e.to_string()))?
            .body_mut()
            .with_config()
            .limit(100 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(bytes))
            .map_err(|e| PyIOError::new_err(e.to_string()))?;
        workbook
            .worksheet_range(sheet_name)
            .map_err(|e| PyValueError::new_err(e.to_string()))?
    } else {
        let file = File::open(source).map_err(|e| PyIOError::new_err(e.to_string()))?;
        let mut workbook: Xlsx<_> = Xlsx::new(BufReader::new(file))
            .map_err(|e| PyIOError::new_err(e.to_string()))?;
        workbook
            .worksheet_range(sheet_name)
            .map_err(|e| PyValueError::new_err(e.to_string()))?
    };

    let mut result: Vec<Value> = Vec::new();

    for (row, cells) in range.rows().enumerate() {
        for (col, cell) in cells.iter().enumerate() {
            let value = match cell {
                Data::String(v) => v.clone(),
                Data::Float(v) => v.to_string(),
                Data::Int(v) => v.to_string(),
                Data::Bool(v) => v.to_string(),
                Data::DateTime(v) => v.to_string(),
                Data::DateTimeIso(v) => v.clone(),
                Data::DurationIso(v) => v.clone(),
                Data::Empty => continue,
                Data::Error(v) => format!("{v:?}"),
            };

            result.push(json!({
                "row": row,
                "col": col,
                "value": value
            }));
        }
    }

    serde_json::to_string(&result).map_err(|e| PyValueError::new_err(e.to_string()))
}