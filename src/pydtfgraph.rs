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


/// A data structure to compute distance-constrained transitive fraternal augmentations.
/// 
/// *TODO* Documentation
#[pyclass(name="DTFGraph",module="platypus",text_signature="($self)")]
pub struct PyDTFGraph {
    pub(crate) G: DTFGraph
}

#[pymethods]
impl PyDTFGraph {
    /// Creates a dtf-augmentation with depth one from the given `graph`.
    /// 
    /// This method takes the provided graph, computes a degeneracy ordering
    /// from it (iteratively deleting vertices of minimum degree) and then
    /// orients all edges according to this ordering.
    #[staticmethod]
    #[pyo3(text_signature="(graph)")]
    pub fn orient(other:&PyEditGraph) -> PyResult<PyDTFGraph> {
        let G = DTFGraph::orient(&other.G);
        Ok(PyDTFGraph{ G })
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

    /// Returns whether vertices `u` and `v` are connected by an edge.
    #[pyo3(text_signature="($self,u,v,/)")]    
    pub fn adjacent(&self, u:Vertex, v:Vertex) -> PyResult<bool> {
        Ok(self.G.adjacent(&u, &v))
    }

    /// Returns the number of edges incident to `u` in the graph.
    #[pyo3(text_signature="($self,u,/)")]    
    pub fn degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.degree(&u))
    }

    #[pyo3(text_signature="($self,u,/)")]    
    pub fn in_degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.in_degree(&u))
    }    

    #[pyo3(text_signature="($self,u,/)")]    
    pub fn out_degree(&self, u:Vertex) -> PyResult<u32> {
        Ok(self.G.out_degree(&u))
    }      
    
    /// Returns the degrees of all vertices in the graph as a [`VMap`].    
    #[pyo3(text_signature="($self,/)")]    
    pub fn degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.degrees()))
    }

    /// Returns the in-degrees of all vertices in the graph as a [VMap].    
    #[pyo3(text_signature="($self,/)")]    
    pub fn in_degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.in_degrees()))
    }    

    /// Returns the out-degrees of all vertices in the graph as a [VMap].    
    #[pyo3(text_signature="($self,/)")]    
    pub fn out_degrees(&self) -> PyResult<PyVMap> {
        Ok(PyVMap::new_int(self.G.out_degrees()))
    }        

    /// Returns whether the vertex `u` is contained in the graph.
    #[pyo3(text_signature="($self,u,/)")]    
    pub fn contains(&mut self, u:Vertex) -> PyResult<bool> {
        Ok(self.G.contains(&u))
    }

    /// Returns the vertices of this graph as a set.
    #[pyo3(text_signature="($self,/)")]    
    pub fn vertices(&self) -> PyResult<VertexSet> {
        Ok(self.G.vertices().cloned().collect())
    }

    /// Returns the edges of this graph as a list.
    #[pyo3(text_signature="($self,/)")]    
    pub fn edges(&self) -> PyResult<Vec<Edge>> {
        Ok(self.G.edges().collect())
    }

    /// Increases the depth of the augmentation.
    /// 
    /// ### Arguments
    /// - `depth` - The target depth of the augmentation
    /// - `frat_depth` - An optimisation parameter. Larger values increase computation time, recommended values are 2 or 1. 
    #[pyo3(text_signature="($self,depth,/,frat_depth=2)")]  
    #[args(frat_depth=2)]
    pub fn augment(&mut self, depth:usize, frat_depth:usize) -> PyResult<()> {   
        self.G.augment(depth, frat_depth);
        Ok(())
    }

    /// Computes an approximate $r$-dominating set of the underlying graph,
    /// where $r$ is the provided `radius`.
    /// 
    /// Note that if `radius` is larger than the current augmentation depth
    /// then graph is augmented further until the depth matches the `radius`.
    /// 
    /// The approximation ratio is a function of the graph's 'sparseness'. 
    /// See \[Dvořák13\] and \[Reidl16\] for the underlying theory and \[Brown20\] for the details of this 
    /// specific version of the algorithm.
    /// 
    /// > \[Dvořák13\]
    /// Dvořák, Z. (2013). Constant-factor approximation of the domination number in sparse graphs. *European Journal of Combinatorics*, 34(5), 833-840.
    /// >
    /// > \[Brown20\]
    /// Brown, C.T., Moritz, D., O’Brien, M.P., Reidl, F., Reiter, T. and Sullivan, B.D., 2020. Exploring neighborhoods in large metagenome assembly graphs using spacegraphcats reveals hidden sequence diversity. *Genome biology*, 21(1), pp.1-16.
    /// >
    /// >\[Reidl16\]
    /// >Reidl, F. (2016). Structural sparseness and complex networks (No. RWTH-2015-07564). Fachgruppe Informatik.    
    #[pyo3(text_signature="($self,r,/)")]    
    pub fn domset(&mut self, radius:u32) -> PyResult<VertexSet> {
        Ok(self.G.domset(radius))
    }

    /// Returns the distance between the vertices `u` and `v` if it
    /// it smaller than the depth of the augmentation. Otherwise returns `None`.    
    #[pyo3(text_signature="($self,u,v/)")]    
    pub fn small_distance(&self, u:Vertex, v:Vertex) -> PyResult<Option<u32>> {
        Ok(self.G.small_distance(&u, &v))
    }
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