#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]

use graphbench::editgraph::EditGraph;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::exceptions::*;
use pyo3::ToPyObject;
use pyo3::types::PyTuple;

pub mod pygraph;
pub mod pyordgraph;
pub mod pydtfgraph;
mod vmap;
mod ducktype;

use graphbench::graph::{Vertex, Graph};
use graphbench::iterators::*;

use crate::pygraph::*;
use crate::pyordgraph::*;
use crate::pydtfgraph::*;
use crate::ducktype::*;

use crate::vmap::*;

/// Returns the vertices of a graph.
#[pyfunction]
#[pyo3(text_signature="(graph)")]
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

/// Returns the edges of a graph.
#[pyfunction]
#[pyo3(text_signature="(graph)")]
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

/// Generates a complete k-partite graph.
/// 
/// Expects as input a sequence of integers which correspond to the sizes of the
/// partite sets. For example, `K(5)` will generate a $K_5$ (a clique on five vertices) or
/// `K(2,5)` a $K_{2,5}$ (a biclique with two vertices on one side and five on the other).
/// 
/// - **\*args:** A list of integers specifying the size of the partite sets.
#[pyfunction(args="*")]
#[pyo3(text_signature="(*args)")]
pub fn K(args: &PyTuple) -> PyResult<PyEditGraph> {
    let parts:Vec<u32> = args.extract()?;
    let res = PyEditGraph::wrap( EditGraph::complete_kpartite(&parts) );
    
    Ok(res)
}

/// Generates a path graph with `n` vertices
#[pyfunction]
#[pyo3(text_signature="(n)")]
pub fn P(n:u32) -> PyResult<PyEditGraph> {
    let res = PyEditGraph::wrap( EditGraph::path(n) );
    
    Ok(res)
}


/// Generates a cycle graph with `n` vertices
#[pyfunction]
#[pyo3(text_signature="(n)")]
pub fn C(n:u32) -> PyResult<PyEditGraph> {
    let res = PyEditGraph::wrap( EditGraph::cycle(n) );
    
    Ok(res)
}



/// Generates a star graph with `n` leaves
#[pyfunction]
#[pyo3(text_signature="(n)")]
pub fn S(n:u32) -> PyResult<PyEditGraph> {
    let res = PyEditGraph::wrap( EditGraph::star(n) );
    
    Ok(res)
}


/// Sparse graph analysis library.
#[pymodule]
#[allow(unused_must_use)]
fn platypus(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyVMap>()?;
    m.add_class::<PyEditGraph>()?;
    m.add_class::<PyOrdGraph>()?;
    m.add_class::<PyDTFGraph>()?;
    m.add_function(wrap_pyfunction!(V, m)?)?;
    m.add_function(wrap_pyfunction!(E, m)?)?;
    m.add_function(wrap_pyfunction!(K, m)?)?;
    m.add_function(wrap_pyfunction!(P, m)?)?;
    m.add_function(wrap_pyfunction!(C, m)?)?;
    m.add_function(wrap_pyfunction!(S, m)?)?;

    Ok(())
}
