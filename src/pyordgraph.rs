use fxhash::{FxHashSet, FxHashMap};

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::class::iter::IterNextOutput;
use pyo3::exceptions;

use std::collections::HashSet;

use graphbench::ordgraph::*;
use graphbench::graph::*;
use graphbench::iterators::*;

use crate::ducktype::*;
use crate::vmap::PyVMap;
use crate::pygraph::PyEditGraph;


/*
    Python methods
*/
#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pymethods]
impl PyOrdGraph {
    pub fn num_vertices(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    #[staticmethod]
    pub fn by_degeneracy(other: &PyEditGraph ) -> PyResult<PyOrdGraph> {
        Ok(PyOrdGraph{G: OrdGraph::by_degeneracy(&other.G)})
    }

    pub fn order(&self) -> PyResult<Vec<Vertex>> {
        Ok(self.G.vertices().cloned().collect())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("OrdGraph (n={},m={})]", self.G.num_vertices(), self.G.num_edges() ))
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

    pub fn left_degree(&self, u:Vertex) -> PyResult<usize> {
        Ok(self.G.left_degree(&u))
    }    

    pub fn right_degree(&self, u:Vertex) -> PyResult<usize> {
        Ok(self.G.right_degree(&u))
    }      
    
    pub fn degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.degrees()))
    }

    pub fn left_degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.left_degrees()))
    }    

    pub fn right_degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.right_degrees()))
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


    /*
        Neighbourhood methods
    */
    pub fn neighbours(&self, u:Vertex) -> PyResult<VertexSet> {
        Ok(self.G.neighbours(&u).cloned().collect())
    }

    pub fn neighbourhood(&self, c:HashSet<u32>) -> PyResult<VertexSet> {
        Ok(self.G.neighbourhood(c.iter()))
    }

    pub fn closed_neighbourhood(&self, c:HashSet<u32>) -> PyResult<VertexSet> {
        Ok(self.G.closed_neighbourhood(c.iter()))
    }

    pub fn r_neighbours(&self, u:Vertex, r:usize) -> PyResult<VertexSet> {
        Ok(self.G.r_neighbours(&u, r))
    }

    /*
        Ordgraph specific methods
    */
    pub fn swap(&mut self, u:Vertex, v:Vertex) -> PyResult<()> {
        self.G.swap(&u,&v);
        Ok(())
    }

    pub fn wreach_sets(&self, r:u32) -> PyResult<VertexMap<VertexMap<u32>>> {
        Ok(self.G.wreach_sets(r))
    }

    pub fn wreach_sizes(&self, r:u32) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.wreach_sizes(r)))
    }    
}

#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pyclass(name="OrdGraph")]
pub struct PyOrdGraph {
    pub(crate) G: OrdGraph
}

impl AttemptCast for PyOrdGraph {
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