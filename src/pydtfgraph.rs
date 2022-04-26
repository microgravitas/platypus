use graphbench::graph::DTFGraph;


/*
    Python methods
*/
#[cfg(not(test))] // pyclass and pymethods break `cargo test`
#[pymethods]
impl PyDTFGraph {
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