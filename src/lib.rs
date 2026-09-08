use calamine::{Data, Reader, Xlsx};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use serde_json::{json, Value};
use std::fs::File;
use std::io::BufReader;

#[pyfunction]
fn read_sheet(path: &str, sheet_name: &str) -> PyResult<String> {
    let file = File::open(path)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;

    let mut workbook: Xlsx<BufReader<File>> =
        Xlsx::new(BufReader::new(file))
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

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

    serde_json::to_string(&result)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pymodule]
mod spelling_checker {
    use super::*;

    #[pyfunction]
    fn read_sheet(path: &str, sheet_name: &str) -> PyResult<String> {
        super::read_sheet(path, sheet_name)
    }
}