#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

use graphbench::editgraph::EditGraph;
use pyo3::prelude::*;
// use pyo3::wrap_pyfunction;

pub mod pygraph;
pub mod pyordgraph;
pub mod pydtfgraph;
mod vmap;
mod ducktype;

use pyo3::exceptions::*;
use pyo3::ToPyObject;
use pyo3::types::PyTuple;

use graphbench::graph::{Vertex, Graph};
use graphbench::iterators::*;

use crate::pygraph::*;
use crate::pyordgraph::*;
use crate::pydtfgraph::*;
use crate::ducktype::*;

use crate::vmap::*;

#[pyfunction]
pub fn V(obj: &PyAny) -> PyResult<Vec<Vertex>> {
    // Since python-facing functions cannot use generic traits, we 
    // have to implement this for every graph type in the crate.
    let res = PyEditGraph::try_cast(obj, |pygraph| -> PyResult<Vec<Vertex>> {
        Ok(pygraph.G.vertices().cloned().collect())
    });

    return_some!(res);

    let res = PyOrdGraph::try_cast(obj, |pygraph| -> PyResult<Vec<Vertex>> {
        Ok(pygraph.G.vertices().cloned().collect())
    });

    return_some!(res);

    Err(PyTypeError::new_err( format!("{:?} is not a graph", obj) ))
}

#[pyfunction]
pub fn E(obj: &PyAny) -> PyResult<Vec<(Vertex,Vertex)>> {
    // Since python-facing functions cannot use generic traits, we 
    // have to implement this for every graph type in the crate.
    let res = PyEditGraph::try_cast(obj, |pygraph| -> PyResult<Vec<(Vertex,Vertex)>> {
        Ok(pygraph.G.edges().collect())
    });

    return_some!(res);

    let res = PyOrdGraph::try_cast(obj, |pygraph| -> PyResult<Vec<(Vertex,Vertex)>> {
        Ok(pygraph.G.edges().collect())
    });

    return_some!(res);

    Err(PyTypeError::new_err( format!("{:?} is not a graph", obj) ))
}

#[pyfunction(args="*")]
pub fn K(args: &PyTuple) -> PyResult<PyEditGraph> {
    let parts:Vec<u32> = args.extract()?;
    let res = PyEditGraph::wrap( EditGraph::complete_kpartite(&parts) );
    
    Ok(res)
}

#[pyfunction]
pub fn P(n:u32) -> PyResult<PyEditGraph> {
    let res = PyEditGraph::wrap( EditGraph::path(n) );
    
    Ok(res)
}

#[pyfunction]
pub fn C(n:u32) -> PyResult<PyEditGraph> {
    let res = PyEditGraph::wrap( EditGraph::cycle(n) );
    
    Ok(res)
}


#[pyfunction]
pub fn S(n:u32) -> PyResult<PyEditGraph> {
    let res = PyEditGraph::wrap( EditGraph::star(n) );
    
    Ok(res)
}


#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pymodule]
fn platypus(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVMap>()?;
    m.add_class::<pygraph::PyEditGraph>()?;
    m.add_class::<pyordgraph::PyOrdGraph>()?;
    m.add_class::<pydtfgraph::PyDTFGraph>()?;
    m.add_wrapped(wrap_pyfunction!(V))?;
    m.add_wrapped(wrap_pyfunction!(E))?;
    m.add_wrapped(wrap_pyfunction!(K))?;
    m.add_wrapped(wrap_pyfunction!(P))?;
    m.add_wrapped(wrap_pyfunction!(C))?;
    m.add_wrapped(wrap_pyfunction!(S))?;

    Ok(())
}
