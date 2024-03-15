use pyo3::prelude::*;
use std::fs;

/// Takes no arguments, returns no value, just prints the greeting.
#[pyfunction]
fn greeting() {
    println!("✨ Hello, world!");
}

/// Takes a filename and returns its content in reverse.
#[pyfunction]
fn tac(filename: &str) -> PyResult<String> {
    let text = fs::read_to_string(&filename)?
        .chars()
        .rev()
        .collect::<String>();
    println!("'{filename}' has been read, returning its content, reversed");
    Ok(text)
}

/// A Python module implemented in Rust.
#[pymodule]
fn sample(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(greeting, m)?)?;
    m.add_function(wrap_pyfunction!(tac, m)?)?;
    Ok(())
}
