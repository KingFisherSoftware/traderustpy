use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;

mod rusty;

/// Returns the number of lines in a given file.
#[pyfunction]
fn count_file_lines(path: &str) -> PyResult<usize> {
    rusty::count_file_lines(path)
        .map_err(|e| PyIOError::new_err(format!("{}", e)))
}

/// Parses a supply level string into a tuple of ints (units, level).
#[pyfunction]
fn parse_supply_level(reading: &str) -> PyResult<(i32, i32)> {
    rusty::parse_supply_level(reading)
        .map_err(|e| PyValueError::new_err(format!("{}: {}", e, reading)))
}

#[pyfunction]
fn stellar_grid_key(x: f64, y: f64, z: f64) -> u64 {
    rusty::stellar_grid_key(x, y, z)
}

/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "traderusty")]
fn traderusty(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(count_file_lines, m)?)?;
    m.add_function(wrap_pyfunction!(parse_supply_level, m)?)?;
    m.add_function(wrap_pyfunction!(stellar_grid_key, m)?)?;
    Ok(())
}
