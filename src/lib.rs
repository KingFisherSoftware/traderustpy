use pyo3::exceptions::{PyIOError};
use pyo3::prelude::*;

mod rusty;

/// Returns the number of lines in a given file.
#[pyfunction]
fn count_file_lines(path: &str) -> PyResult<usize> {
        rusty::count_file_lines(path)
            .map_err(|e| PyIOError::new_err(format!("{}", e)))
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "traderusty")]
fn traderusty(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(count_file_lines, m)?)?;
    Ok(())
}
