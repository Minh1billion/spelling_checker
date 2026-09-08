use pyo3::prelude::*;

mod io;
mod tagger;
mod tokenizer;

#[pymodule]
mod spelling_checker {
    use super::*;

    #[pyfunction]
    fn read_sheet(source: &str, sheet_name: &str) -> PyResult<String> {
        io::read_sheet(source, sheet_name)
    }

    #[pyfunction]
    fn tokenize(text: &str) -> Vec<String> {
        tokenizer::tokenize(text)
    }
}