use algo::{multiply, Matrix};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::fmt;

#[napi(js_name = "Matrix")]
#[derive(Debug)]
pub struct JsMatrix {
  inner: Matrix<f64>,
}

#[napi]
impl JsMatrix {
  #[napi(constructor)]
  pub fn new(data: Vec<Vec<f64>>, _env: Env) -> Result<Self> {
    if data.is_empty() {
      let err = Error::new(Status::InvalidArg, "Matrix must have at least one row");
      return Err(err);
    }

    let rows = data.len();
    let cols = data[0].len();
    let data: Vec<_> = data.into_iter().flatten().collect();
    Ok(Self {
      inner: Matrix::new(data, rows, cols),
    })
  }

  #[napi]
  pub fn mul(&self, other: &JsMatrix) -> Result<Self> {
    let result = multiply(&self.inner, &other.inner).unwrap();
    Ok(JsMatrix { inner: result })
  }

  #[napi]
  pub fn multiply(&self, other: Vec<Vec<f64>>, env: Env) -> Result<Self> {
    let other = JsMatrix::new(other, env)?;
    self.mul(&other)
  }

  #[napi]
  pub fn display(&self) -> String {
    format!("{}", self.inner)
  }
}

impl fmt::Display for JsMatrix {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self.inner)
  }
}
