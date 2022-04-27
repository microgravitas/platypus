use pyo3::prelude::*;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::*;

use std::collections::HashSet;

use graphbench::graph::*;
use graphbench::iterators::*;
use graphbench::algorithms::*;
use graphbench::dtfgraph::*;

use crate::ducktype::*;
use crate::PyVMap;
use crate::pygraph::PyEditGraph;

/*
    Python methods
*/
#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pymethods]
impl PyDTFGraph {
    #[staticmethod]
    pub fn orient(other:&PyEditGraph) -> PyResult<PyDTFGraph> {
        let G = DTFGraph::orient(&other.G);
        Ok(PyDTFGraph{ G })
    }

    pub fn num_vertices(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("DTFGraph (n={},m={},depth={})]", self.G.num_vertices(), self.G.num_edges(), self.G.get_depth() ))
    }    

    pub fn num_edges(&self) -> PyResult<usize> {
        Ok(self.G.num_edges())
    }

    pub fn adjacent(&self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok(self.G.adjacent(&u, &v))
    }

    pub fn degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.degree(&u))
    }

    pub fn in_degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.in_degree(&u))
    }    

    pub fn out_degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.out_degree(&u))
    }      
    
    pub fn degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.degrees()))
    }

    pub fn in_degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.in_degrees()))
    }    

    pub fn out_degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.out_degrees()))
    }        

    pub fn contains(&mut self, u:Vertex) -> PyResult<bool> {
        Ok(self.G.contains(&u))
    }

    pub fn vertices(&self) -> PyResult<VertexSet> {
        Ok(self.G.vertices().cloned().collect())
    }

    pub fn edges(&self) -> PyResult<Vec<Edge>> {
        Ok(self.G.edges().collect())
    }

    pub fn domset(&mut self, radius:u32) -> PyResult<VertexSet> {
        Ok(self.G.domset(radius))
    }

    pub fn small_distance(&self, u:Vertex, v:Vertex) -> PyResult<Option<u32>> {
        Ok(self.G.small_distance(&u, &v))
    }
}

#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pyclass(name="DTFGraph")]
pub struct PyDTFGraph {
    pub(crate) G: DTFGraph
}

impl AttemptCast for PyDTFGraph {
    fn try_cast<F, R>(obj: &PyAny, f: F) -> Option<R>
    where F: FnOnce(&Self) -> R,
    {
        if let Ok(py_cell) = obj.downcast::<PyCell<Self>>() {
            let map:&Self = &*(py_cell.borrow());  
            Some(f(map))
        } else {
            None
        }
    }
}