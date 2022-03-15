#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

use pyo3::prelude::*;
// use pyo3::wrap_pyfunction;

pub mod pygraph;
pub mod pyordgraph;
mod vmap;
mod ducktype;

use pyo3::exceptions::*;
use pyo3::ToPyObject;

use crate::vmap::*;

#[macro_export]
macro_rules! return_some{
    ($a:ident) => {
        if let Some(obj) = $a {
            return obj;
        }
    }
}

trait AttemptCast {
    fn try_cast<F, R>(obj: &PyAny, f: F) -> Option<R>
    where F: FnOnce(&Self) -> R;
}

#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pymodule]
fn platypus(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVMap>()?;
    m.add_class::<pygraph::PyGraph>()?;
    m.add_class::<pyordgraph::PyOrdGraph>()?;
    // m.add_wrapped(wrap_pyfunction!(from_pid))?;

    Ok(())
}
