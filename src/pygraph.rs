use fxhash::{FxHashSet, FxHashMap};

use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::*;
use pyo3::{PyClass,ToPyObject};
use pyo3::exceptions;

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

/// A versatile graph class which allows different editing operations.
/// 
/// *TODO* Documentation
#[pyclass(name="EditGraph",module="platypus",text_signature="($self,/)")]
pub struct PyEditGraph {
    pub(crate) G: EditGraph
}


impl PyEditGraph {
    pub fn wrap(G: EditGraph) -> PyEditGraph {
        PyEditGraph{G}
    }
}

//
// Python methods for PyEditGraph
//
#[pymethods]
impl PyEditGraph {
    
    /// Creates an empty EditGraph.
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
    #[pyo3(text_signature="($self, odering,/)")]    
    pub fn to_ordered(&self, ordering:Option<Vec<u32>>) -> PyResult<PyOrdGraph> {
        if let Some(ord) = ordering {
            Ok(PyOrdGraph{G: OrdGraph::with_ordering(&self.G, ord.iter())})
        } else {
            Ok(PyOrdGraph{G: OrdGraph::by_degeneracy(&self.G)})
        }
    }

    /// Normalizes the graph by relabelling vertices to 0...n-1. The relative
    /// order of the vertex ids remains the same, e.g. the smallest id is mappet to 0
    /// and the largest to n-1.
    #[pyo3(text_signature="($self,/)")]    
    pub fn normalize(&mut self) -> FxHashMap<Vertex, Vertex>{
        let (GG, mapping) = self.G.normalize();
        self.G = GG;
        mapping
    }

    /// Loads a graph from the provided file. The expected file format is a 
    /// text file which contains the edges of the graph separated by line breaks. Only
    /// integers are supported as vertex names. For example, the following file
    /// ```
    /// 0 1
    /// 1 2
    /// 2 3
    /// ```
    /// Corresponds to a path of length four with vertices 0,1,2,4.
    /// 
    /// This method also accepts gzipped files with this format, the 
    /// file ending must be `.gz`.
    #[staticmethod]
    #[pyo3(text_signature="(filename,/)")]    
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

    /// Computes the degeneracy of the graph. For reasons of efficiency, the degeneracy
    /// is computed exactly if it lies below 32 and otherwise as a 2-approximation.
    /// 
    /// Returns a quadruplet `(lower, upper, order, corenums)` where
    /// 
    ///  - `lower` is a lower bound on the degeneracy
    ///  - `upper` is an upper bound on the degeneracy
    ///  - `order` is the degeneracy ordering with degeneracy `upper`
    ///  - `corenums` is a mapping that provides the core number for every vertex
    #[pyo3(text_signature="($self,/)")]    
    pub fn degeneracy(&self) -> PyResult<(u32, u32, Vec<Vertex>,VertexMap<u32>)> {
        Ok(self.G.degeneracy())
    }

    /// Returns the number of vertices in the graph.
    #[pyo3(text_signature="($self,/)")]
    pub fn num_vertices(&self) -> PyResult<usize> {
        Ok(self.G.num_vertices())
    }

    /// Returns the number of edges in the graph.
    #[pyo3(text_signature="($self,/)")]    
    pub fn num_edges(&self) -> PyResult<usize> {
        Ok(self.G.num_edges())
    }

    /// Returns whether the vertices `u` and `v` are adjacent in the graph,
    /// that is, whether or not the edge `u`,`v` is contained in it.
    #[pyo3(text_signature="($self, u, v,/)")]    
    pub fn adjacent(&self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok(self.G.adjacent(&u, &v))
    }

    /// Returns the number of neighbours `u` has in the graph.
    #[pyo3(text_signature="($self, u,/)")]    
    pub fn degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.degree(&u))
    }

    /// Returns the degree of every vertex in the form of a [`VMap`](PyVMap).
    #[pyo3(text_signature="($self,/)")]
    pub fn degrees(&self) -> PyResult<PyVMap> {
        let degs = self.G.degrees().iter().map(|(k,v)| (*k, *v as i32)).collect();
        Ok(PyVMap::new_int(degs))
    }

    /// Returns whether `u` is a vertex in the graph.
    #[pyo3(text_signature="($self, u,/)")]    
    pub fn contains(&mut self, u:Vertex) -> PyResult<bool> {
        Ok(self.G.contains(&u))
    }

    /// Returns the set of vertices of this graph.
    #[pyo3(text_signature="($self,/)")]    
    pub fn vertices(&self) -> PyResult<VertexSet> {
        Ok(self.G.vertices().cloned().collect())
    }

    /// Returns a list of edges contained in this graph.
    #[pyo3(text_signature="($self,/)")]
    pub fn edges(&self) -> PyResult<Vec<Edge>> {
        Ok(self.G.edges().collect())
    }


    /// Returns the neigbhours of the vertex `u`.
    #[pyo3(text_signature="($self, u,/)")]
    pub fn neighbours(&self, u:Vertex) -> PyResult<VertexSet> {
        Ok(self.G.neighbours(&u).cloned().collect())
    }

    /// Returns the joint neighbourhood of a collection `vertices`,
    /// that is, all vertices that have a neighbour in `vertices`
    /// but are not themselves contained in it.
    #[pyo3(text_signature="($self, vertices,/)")]
    pub fn neighbourhood(&self, vertices:&PyAny) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.neighbourhood(vertices.iter()))
    }

    /// Returns the joint closed neighbourhood of a collection `vertices`,
    /// that is, all vertices that have at least one neighbour `vertices`
    /// or are themselves contained in it.
    #[pyo3(text_signature="($self, vertices,/)")]
    pub fn closed_neighbourhood(&self, vertices:&PyAny) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.closed_neighbourhood(vertices.iter()))
    }

    /// Returns all vertices that have distance at most `r` to `u`.
    #[pyo3(text_signature="($self, u, r,/)")]
    pub fn r_neighbours(&self, u:Vertex, r:usize) -> PyResult<VertexSet> {
        Ok(self.G.r_neighbours(&u, r))
    }

    /// Returns all vertices that have distance at most `r` to some vertex
    /// in the provided collection.
    #[pyo3(text_signature="($self, vertices, r,/)")]
    pub fn r_neighbourhood(&self, vertices:&PyAny, r:usize) -> PyResult<VertexSet> {
        let vertices = to_vertex_list(vertices)?;
        Ok(self.G.r_neighbourhood(vertices.iter(), r))
    }    

    /// Adds the vertex `u` to the graph.
    #[pyo3(text_signature="($self, u,/)")]
    pub fn add_vertex(&mut self, u:Vertex) -> PyResult<()> {
        self.G.add_vertex(&u);
        Ok(())
    }

    /// Adds the edge `u`,`v` to the graph. 
    #[pyo3(text_signature="($self, u, v,/)")]
    pub fn add_edge(&mut self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok( self.G.add_edge(&u, &v) )
    }

    /// Removes the edges `u`,`v` from the graph.
    #[pyo3(text_signature="($self, u, v,/)")]
    pub fn remove_edge(&mut self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok( self.G.remove_edge(&u, &v) )
    }

    /// Removes the vertex `u` from the graph. All edges incident to `u`
    /// are also removed.
    #[pyo3(text_signature="($self, u,/)")]
    pub fn remove_vertex(&mut self, u:Vertex) -> PyResult<bool> {
        Ok( self.G.remove_vertex(&u) )
    }

    /// Removes all loops from the graph, meaning all edges where both
    /// endpoints are the same vertex.
    #[pyo3(text_signature="($self,/)")]
    pub fn remove_loops(&mut self) -> PyResult<usize> {
        Ok( self.G.remove_loops() )
    }

    /// Removes all vertices from the graph that have are not incident
    /// to any edge.
    #[pyo3(text_signature="($self,/)")]
    pub fn remove_isolates(&mut self) -> PyResult<usize> {
        Ok( self.G.remove_isolates() )
    }

    
    /// Contracts the provided collection of vertices into a single new vertex
    /// and returns that vertex. The resulting neighbourhood of the vertex is the
    /// union of all neighbourhoods of the provided vertices.
    #[pyo3(text_signature="($self, vertices,/)")]    
    pub fn contract(&mut self, vertices:&PyAny) -> PyResult<Vertex> {
        let vertices = to_vertex_list(vertices)?;
        Ok( self.G.contract(vertices.iter()) )
    }

    /// Similar to `EditGraph.contract`, but contracts `vertices`
    /// into the specified vertex `center`.
    #[pyo3(text_signature="($self, center, vertices,/)")]
    pub fn contract_into(&mut self, center:Vertex, vertices:&PyAny) -> PyResult<()> {
        let vertices = to_vertex_list(vertices)?;
        self.G.contract_into(&center, vertices.iter());
        Ok(())
    }

    /// Contracts a pair `u`, `v` into `u`. This method works regardless of whether
    /// `uv` is an edge in the graph or not.
    #[pyo3(text_signature="($self, u, v,/)")]
    pub fn contract_pair(&mut self, u:Vertex, v:Vertex) -> PyResult<()> {
        Ok(self.G.contract_pair(&u, &v)) 
    }


    /// Creates a copy of the graph.
    #[pyo3(text_signature="($self,/)")]
    pub fn copy(&self) -> PyResult<PyEditGraph> {
        Ok(PyEditGraph{G: self.G.clone()})
    }

    // This allows us to use the shorthand `G[X]` for `G.subgraph(X)`
    pub fn __getitem__(&self, obj:&PyAny) -> PyResult<PyEditGraph> {
        self.subgraph(obj)
    }

    /// Creates a subgraph on the provided vertex collection. This method
    /// also accepts a [PyVMapBool](VMapBool), in this case all vertices that
    /// map to `True` are used.
    #[pyo3(text_signature="($self, collection,/)")]
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

    /// Returns a list of this graph's connected components .
    #[pyo3(text_signature="($self,/)")]
    pub fn components(&self) -> PyResult<Vec<VertexSet>> {
        Ok(self.G.components())
    }

    /// Tests whether the graph is bipartite.
    /// 
    /// Returns a tuple `(bip, witness)` where `bip` is a boolean that indicates
    /// whether the graph is bipartite and `witness` is either a bipartition or an odd cycle.
    #[pyo3(text_signature="($self,/)")] 
    pub fn is_bipartite(&self) -> PyResult<(bool, PyObject)> {
        let res = self.G.is_bipartite();

        let gil = Python::acquire_gil();
        let py = gil.python();

        // Ok(pyref.to_object(py))        
        let (bipartite, witness) = match res {
            BipartiteWitness::Bipartition(left, right) => (true, (left.to_object(py), right.to_object(py)).to_object(py)),
            BipartiteWitness::OddCycle(cycle) => (false, cycle.to_object(py))
        };

        Ok((bipartite, witness))
    }

    /// Computes the disjoint union of the two graphs.
    pub fn __add__(&self, other: &PyEditGraph) -> PyResult<PyEditGraph> {
        Ok(PyEditGraph{G: self.G.disj_union(&other.G)})
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



