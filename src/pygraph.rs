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
        Ok(format!("EditGraph (n={},m={})", self.G.num_vertices(), self.G.num_edges() ))
    }    

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    
    /// Converts the graph into an [`OrdGraph`](PyOrdGraph) either by using the
    /// optional `ordering` or by computing a degeneracy ordering of the graph.
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
    #[pyo3(text_signature = "(filename/)")]
    /// Loads a graph from the provided file. The expected file format is a 
    /// text file which contains the edges of the graph separated by lines. Only
    /// integers are supported as vertex names. For example, the following file
    /// ```
    /// 0 1
    /// 1 2
    /// 2 3
    /// ```
    /// Corresponds to a path of length four with vertices 0,1,2,4.
    /// 
    /// This method also accepts gzipped versions of this format, however the 
    /// file ending must be `.gz` in these cases.
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

    pub fn degeneracy(&self) -> PyResult<(u32, u32, Vec<Vertex>,VertexMap<u32>)> {
        Ok(self.G.degeneracy())
    }

    /// Returns the number of vertices in the graph.
    pub fn num_vertices(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    /// Returns the number of edges in the graph.
    pub fn num_edges(&self) -> PyResult<usize> {
        Ok(self.G.num_edges())
    }

    /// Returns whether the vertices `u` and `v` are adjacent in the graph,
    /// that is, whether or not the edge `u`,`v` is contained in it.
    pub fn adjacent(&self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok(self.G.adjacent(&u, &v))
    }

    /// Returns the number of neighbours `u` has in the graph.
    pub fn degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.degree(&u))
    }

    /// Returns the degree of every vertex in the form of a [`VMap`](PyVMap).
    pub fn degrees(&self) -> PyResult<PyVMap> {
        let degs = self.G.degrees().iter().map(|(k,v)| (*k, *v as i32)).collect();
        Ok(PyVMap::new_int(degs))
    }

    /// Returns whether `u` is a vertex in the graph.
    pub fn contains(&mut self, u:Vertex) -> PyResult<bool> {
        Ok(self.G.contains(&u))
    }

    /// Returns the set of vertices of this graph.
    pub fn vertices(&self) -> PyResult<VertexSet> {
        Ok(self.G.vertices().cloned().collect())
    }

    /// Returns a list of edges contained in this graph.
    pub fn edges(&self) -> PyResult<Vec<Edge>> {
        Ok(self.G.edges().collect())
    }


    /*
        Neighbourhood methods
    */
    /// Returns the neigbhours of the vertex `u`.
    pub fn neighbours(&self, u:Vertex) -> PyResult<VertexSet> {
        Ok(self.G.neighbours(&u).cloned().collect())
    }

    /// Returns the joint neighbourhood of a collection of vertices,
    /// that is, all vertices that have a neighbour in the provided collection
    /// but are not themselves contained in it.
    pub fn neighbourhood(&self, vertices:&PyAny) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.neighbourhood(vertices.iter()))
    }

    /// Returns the joint closed neighbourhood of a collection of vertices,
    /// that is, all vertices that have at least one neighbour in the provided
    /// collection.
    pub fn closed_neighbourhood(&self, vertices:&PyAny) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.closed_neighbourhood(vertices.iter()))
    }

    /// Returns all vertices that have distance at most `r` to `u`.
    pub fn r_neighbours(&self, u:Vertex, r:usize) -> PyResult<VertexSet> {
        Ok(self.G.r_neighbours(&u, r))
    }

    /// Returns all vertices that have distance at most `r` to some vertex
    /// in the provided collection.
    pub fn r_neighbourhood(&self, vertices:&PyAny, r:usize) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.r_neighbourhood(vertices.iter(), r))
    }    

    /// Adds the vertex `u` to the graph.
    pub fn add_vertex(&mut self, u:Vertex) -> PyResult<()> {
        self.G.add_vertex(&u);
        Ok(())
    }

    /// Adds the edge `u`,`v` to the graph. 
    pub fn add_edge(&mut self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok( self.G.add_edge(&u, &v) )
    }

    /// Removes the edges `u`,`v` from the graph.
    pub fn remove_edge(&mut self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok( self.G.remove_edge(&u, &v) )
    }

    /// Removes the vertex `u` from the graph. All edges incident to `u`
    /// are also removed.
    pub fn remove_vertex(&mut self, u:Vertex) -> PyResult<bool> {
        Ok( self.G.remove_vertex(&u) )
    }

    /// Removes all loops from the graph, meaning all edges where both
    /// endpoints are the same vertex.
    pub fn remove_loops(&mut self) -> PyResult<usize> {
        Ok( self.G.remove_loops() )
    }

    /// Removes all vertices from the graph that have are not incident
    /// to any edge.
    pub fn remove_isolates(&mut self) -> PyResult<usize> {
        Ok( self.G.remove_isolates() )
    }

    /*
        Advanced operations
    */
    /// Contracts the provided collection of vertices into a single new vertex
    /// and returns that vertex. The resulting neighbourhood of the vertex is the
    /// union of all neighbourhoods of the provided vertices.
    pub fn contract(&mut self, vertices:&PyAny) -> PyResult<Vertex> {
        let vertices = to_vertex_list(vertices)?;
        Ok( self.G.contract(vertices.iter()) )
    }

    /// Similar to [PyEditGraph::contract](EditGraph::contract), but we can specify
    /// what the resultant vertex is supposed to be.
    pub fn contract_into(&mut self, center:Vertex, vertices:&PyAny) -> PyResult<()> {
        let vertices = to_vertex_list(vertices)?;
        self.G.contract_into(&center, vertices.iter());
        Ok(())
    }

    /*
        Subgraphs and components
    */
    /// Creates a copy of the graph.
    pub fn copy(&self) -> PyResult<PyEditGraph> {
        Ok(PyEditGraph{G: self.G.clone()})
    }

    pub fn __getitem__(&self, obj:&PyAny) -> PyResult<PyEditGraph> {
        self.subgraph(obj)
    }

    /// Creates a subgraph on the provided vertex collection. This method
    /// also accepts a [PyVMapBool](VMapBool), in this case all vertices that
    /// map to `True` are used.
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

    /// Returns a list of connected components.
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



