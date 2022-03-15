use pyo3::prelude::*;
// use pyo3::wrap_pyfunction;

pub mod pygraph;
pub mod pyordgraph;

#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pymodule]
fn platypus(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<pygraph::PyGraph>()?;
    m.add_class::<pygraph::PyVertexMapDegree>()?;
    m.add_class::<pygraph::PyVertexMapBool>()?;
    m.add_class::<pyordgraph::PyOrdGraph>()?;
    // m.add_wrapped(wrap_pyfunction!(from_pid))?;

    Ok(())
}
