use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct AmapMetrics {
    data: Arc<HashMap<&'static str, AtomicI64>>,
}

impl AmapMetrics {
    pub fn new(metric_names: &[&'static str]) -> Self {
        let map = metric_names
            .iter()
            .map(|&name| (name, AtomicI64::new(0)))
            .collect();
        Self {
            data: Arc::new(map),
        }
    }

    pub fn inc(&self, key: impl AsRef<str>) -> Result<()> {
        let key = key.as_ref();
        let counter = self.data.get(key).ok_or_else(|| anyhow!("Key not found"))?;
        counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn dec(&self, key: impl AsRef<str>) -> Result<()> {
        let key = key.as_ref();
        let counter = self.data.get(key).ok_or_else(|| anyhow!("Key not found"))?;
        counter.fetch_sub(1, Ordering::Relaxed);
        Ok(())
    }
}

impl Clone for AmapMetrics {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

impl fmt::Display for AmapMetrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
