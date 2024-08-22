use pyo3::prelude::*;
use roaring::{MultiOps, RoaringBitmap};
use std::fmt;

#[pyclass(name = "Bitmap")]
pub struct PyBitmap {
    inner: RoaringBitmap,
}

#[pymethods]
impl PyBitmap {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: RoaringBitmap::new(),
        }
    }

    pub fn insert(&mut self, value: u32) {
        self.inner.insert(value);
    }

    pub fn contains(&self, value: u32) -> bool {
        self.inner.contains(value)
    }

    pub fn remove(&mut self, value: u32) {
        self.inner.remove(value);
    }

    pub fn len(&self) -> usize {
        self.inner.iter().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.inner.is_disjoint(&other.inner)
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.inner.is_subset(&other.inner)
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        self.inner.is_superset(&other.inner)
    }

    pub fn union(&self, other: &PyBitmap) -> Self {
        let bitmaps = vec![&self.inner, &other.inner];
        let iter = bitmaps.union();
        Self { inner: iter }
    }

    pub fn intersection(&self, other: &PyBitmap) -> Self {
        let bitmaps = vec![&self.inner, &other.inner];
        let iter = bitmaps.intersection();
        Self { inner: iter }
    }

    pub fn difference(&self, other: &PyBitmap) -> Self {
        let bitmaps = vec![&self.inner, &other.inner];
        let iter = bitmaps.difference();
        Self { inner: iter }
    }

    pub fn symmetric_difference(&self, other: &PyBitmap) -> Self {
        let bitmaps = vec![&self.inner, &other.inner];
        let iter = bitmaps.symmetric_difference();
        Self { inner: iter }
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn __repr__(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for PyBitmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
