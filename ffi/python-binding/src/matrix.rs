use algo::{multiply, Matrix};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fmt;

#[pyclass(name = "Matrix")]
pub struct PyMatrix {
    inner: Matrix<f64>,
}

#[pymethods]
impl PyMatrix {
    #[new]
    pub fn try_new(data: Vec<Vec<f64>>) -> PyResult<Self> {
        if data.is_empty() {
          let err = PyValueError::new_err("Matrix must have at least one row");
          return Err(err);
        }

        let rows = data.len();
        let cols = data[0].len();
        let data: Vec<_> = data.into_iter().flatten().collect();
        Ok(Self {
          inner: Matrix::new(data, rows, cols),
        })
    }

    pub fn mul(&self, other: &PyMatrix) -> PyResult<PyMatrix> {
        let result = multiply(&self.inner, &other.inner).unwrap();
        Ok(PyMatrix { inner: result })
    }

    pub fn multiply(&self, other: Vec<Vec<f64>>) -> PyResult<PyMatrix> {
        let other = PyMatrix::try_new(other)?;
        self.mul(&other)
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn __repr__(&self) -> String {
        self.to_string()
    }

}

impl fmt::Display for PyMatrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}