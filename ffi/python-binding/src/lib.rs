mod hasher;
mod matrix;
mod roaring;

use hasher::PyAlgo;
use matrix::PyMatrix;
use pyo3::prelude::*;
use roaring::PyBitmap;

#[pyfunction]
fn hello() -> PyResult<String> {
    Ok("Hello from python-binding!".into())
}

#[pymodule]
fn _lowlevel(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    m.add_class::<PyAlgo>()?;
    m.add_class::<PyMatrix>()?;
    m.add_class::<PyBitmap>()?;

    Ok(())
}