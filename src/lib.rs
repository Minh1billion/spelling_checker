use pyo3::prelude::*;

mod io;
mod spellcheck;
mod tagger;
mod tokenizer;

#[pymodule]
mod spelling_checker {
    use super::*;
    use std::collections::HashSet;

    #[pyfunction]
    fn read_sheet(source: &str, sheet_name: &str) -> PyResult<String> {
        io::read_sheet(source, sheet_name)
    }

    #[pyfunction]
    fn tokenize(text: &str) -> Vec<String> {
        tokenizer::tokenize(text)
    }

    #[pyfunction]
    fn tag(token: &str) -> Vec<(String, &'static str)> {
        tagger::tag(token)
    }

    #[pyfunction]
    #[pyo3(signature = (token, whitelist, lang="both"))]
    fn check(token: &str, whitelist: Vec<String>, lang: &str) -> bool {
        let whitelist: HashSet<String> = whitelist.into_iter().collect();
        spellcheck::spellcheck(token, &whitelist, lang)
    }
}