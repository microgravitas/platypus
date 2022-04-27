use fxhash::{FxHashSet, FxHashMap};

use pyo3::prelude::*;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::*;

use std::collections::HashSet;

use graphbench::graph::*;
use graphbench::algorithms::*;
use graphbench::ordgraph::*;
use graphbench::editgraph::*;
use graphbench::iterators::*;

use crate::pyordgraph::*;
use crate::vmap::*;
use crate::ducktype::*;
use crate::*;

use std::borrow::Cow;
use std::cell::{Cell, RefCell};



/*
    Helper methods
*/
fn to_vertex_list(obj:&PyAny) -> PyResult<Vec<u32>>  {
    let vec:Vec<_> = obj.iter()?.map(|i| i.and_then(PyAny::extract::<u32>).unwrap()).collect();
    Ok(vec)
}


#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pyclass(name="EditGraph")]
pub struct PyEditGraph {
    pub(crate) G: EditGraph
}


/*
    Python methods
*/
#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pymethods]
impl PyEditGraph {
    #[new]
    pub fn new() -> PyEditGraph {
        PyEditGraph{G: EditGraph::new()}
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("EditGraph (n={},m={})]", self.G.num_vertices(), self.G.num_edges() ))
    }    

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    pub fn to_ordered(&self, ordering:Option<Vec<u32>>) -> PyResult<PyOrdGraph> {
        if let Some(ord) = ordering {
            Ok(PyOrdGraph{G: OrdGraph::with_ordering(&self.G, ord.iter())})
        } else {
            Ok(PyOrdGraph{G: OrdGraph::by_degeneracy(&self.G)})
        }
    }

    pub fn normalize(&mut self) -> FxHashMap<Vertex, Vertex>{
        let (GG, mapping) = self.G.normalize();
        self.G = GG;
        mapping
    }

    #[staticmethod]
    pub fn from_file(filename:&str) -> PyResult<PyEditGraph> {
        if &filename[filename.len()-3..] == ".gz" {
            match EditGraph::from_gzipped(filename) {
                Ok(G) => Ok(PyEditGraph{G}),
                Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>("IO-Error"))
            }
        } else {
            match EditGraph::from_txt(filename) {
                Ok(G) => Ok(PyEditGraph{G}),
                Err(_) => Err(PyErr::new::<exceptions::PyIOError, _>("IO-Error"))
            }
        }
    }

    pub fn degeneracy(&self) -> PyResult<(Vec<Vertex>,VertexMap<u32>)> {
        Ok(self.G.degeneracy())
    }

    pub fn num_vertices(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
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

    pub fn degrees(&self) -> PyResult<PyVMap> {
        let degs = self.G.degrees().iter().map(|(k,v)| (*k, *v as i32)).collect();
        Ok(PyVMap::new_int(degs))
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

    pub fn neighbourhood(&self, vertices:&PyAny) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.neighbourhood(vertices.iter()))
    }

    pub fn closed_neighbourhood(&self, vertices:&PyAny) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.closed_neighbourhood(vertices.iter()))
    }

    pub fn r_neighbours(&self, u:Vertex, r:usize) -> PyResult<VertexSet> {
        Ok(self.G.r_neighbours(&u, r))
    }

    pub fn r_neighbourhood(&self, vertices:&PyAny, r:usize) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.r_neighbourhood(vertices.iter(), r))
    }    

    pub fn add_vertex(&mut self, u:Vertex) -> PyResult<()> {
        self.G.add_vertex(&u);
        Ok(())
    }

    pub fn add_edge(&mut self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok( self.G.add_edge(&u, &v) )
    }

    pub fn remove_edge(&mut self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok( self.G.remove_edge(&u, &v) )
    }

    pub fn remove_vertex(&mut self, u:Vertex) -> PyResult<bool> {
        Ok( self.G.remove_vertex(&u) )
    }

    pub fn remove_loops(&mut self) -> PyResult<usize> {
        Ok( self.G.remove_loops() )
    }

    pub fn remove_isolates(&mut self) -> PyResult<usize> {
        Ok( self.G.remove_isolates() )
    }

    /*
        Advanced operations
    */

    pub fn contract(&mut self, vertices:&PyAny) -> PyResult<Vertex> {
        let vertices = to_vertex_list(vertices)?;
        Ok( self.G.contract(vertices.iter()) )
    }

    pub fn contract_into(&mut self, center:Vertex, vertices:&PyAny) -> PyResult<()> {
        let vertices = to_vertex_list(vertices)?;
        self.G.contract_into(&center, vertices.iter());
        Ok(())
    }

    /*
        Subgraphs and components
    */
    pub fn copy(&self) -> PyResult<PyEditGraph> {
        Ok(PyEditGraph{G: self.G.clone()})
    }

    pub fn __getitem__(&self, obj:&PyAny) -> PyResult<PyEditGraph> {
        self.subgraph(obj)
    }

    pub fn subgraph(&self, obj:&PyAny) -> PyResult<PyEditGraph> {
        let res = PyVMap::try_cast(obj, |map| -> VertexMap<bool> {
            map.to_bool().iter().map(|(k,v)| (*k, *v)).collect()
        });

        if let Some(vmap) = res {
            let it = vmap.iter().filter(|(_,v)| **v).map(|(k,_)| k);
            Ok(PyEditGraph{G: self.G.subgraph( it )} )
        } else {
            let vertices = to_vertex_list(obj)?;
            Ok(PyEditGraph{G: self.G.subgraph(vertices.iter())} )
        }
    }

    pub fn components(&self) -> PyResult<Vec<VertexSet>> {
        Ok(self.G.components())
    }
}

impl AttemptCast for PyEditGraph {
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



