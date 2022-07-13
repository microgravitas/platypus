#![allow(unused_imports)]

use std::borrow::Cow;

use itertools::join;

use pyo3::prelude::*;
use pyo3::pyclass::CompareOp;
use pyo3::types::*;
use pyo3::PyClass;
use pyo3::exceptions::*;
use pyo3::ToPyObject;

use ordered_float::OrderedFloat;

mod helpers;
use self::helpers::*;
use super::ducktype::*;

use crate::*;

use graphbench::graph::{VertexMap, VertexSet};

#[derive(Clone,Debug)]
pub enum VMapTypes {
    VMINT(VertexMap<i32>),
    VMFLOAT(VertexMap<f32>),
    VMBOOL(VertexMap<bool>)
}

impl From<VertexMap<i32>> for VMapTypes {
    fn from(mp: VertexMap<i32>) -> Self { 
        VMapTypes::VMINT(mp)
    }
}

impl From<VertexMap<f32>> for VMapTypes {
    fn from(mp: VertexMap<f32>) -> Self { 
        VMapTypes::VMFLOAT(mp)
    }
}

impl From<VertexMap<bool>> for VMapTypes {
    fn from(mp: VertexMap<bool>) -> Self { 
        VMapTypes::VMBOOL(mp)
    }
}

/// A map with vertices as keys
/// 
/// *TODO* Documentation
#[derive(Debug)]
#[pyclass(name="VMap",module="platypus",text_signature="($self,map/)")]
pub struct PyVMap {
    pub(crate) contents: VMapTypes
}

impl PyVMap {
    pub fn new(contents: VMapTypes) -> Self  {
        PyVMap{ contents }
    }

    pub fn new_int<T>(contents: VertexMap<T>) -> Self 
        where T: TryInto<i32> + Copy,
        <T as TryInto<i32>>::Error: std::fmt::Debug {
        // We want to be able to pass either u32 or i32 here, hence
        // the generics. The type constraints on <T as TryInto<..> are there
        // so that unwrap() can be called as it uses the Debug trait.            
        let vmap:VertexMap<i32> = contents.iter()
                .map(|(k,v)| (*k, TryInto::<i32>::try_into(*v).unwrap() ))
                .collect();
        PyVMap{ contents: VMapTypes::VMINT(vmap) }
    }

    pub fn new_float(contents: VertexMap<f32>) -> Self  {
        PyVMap{ contents: VMapTypes::VMFLOAT(contents) }
    }

    pub fn new_bool(contents: VertexMap<bool>) -> Self  {
        PyVMap{ contents: VMapTypes::VMBOOL(contents) }
    }

    pub fn is_int(&self) -> bool {
        match self.contents {
            VMapTypes::VMINT(_) => true,
            _ => false
        }
    }

    pub fn is_float(&self) -> bool {
        match self.contents {
            VMapTypes::VMFLOAT(_) => true,
            _ => false
        }
    }
    
    pub fn is_bool(&self) -> bool {
        match self.contents {
            VMapTypes::VMBOOL(_) => true,
            _ => false
        }
    }    

    pub fn to_int(&self) -> Cow<VertexMap<i32>> {
        use VMapTypes::*;
        match &self.contents {
            VMINT(vmap) => Cow::Borrowed(vmap),
            VMFLOAT(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, *v as i32) ).collect();
                Cow::Owned( res )
            },
            VMBOOL(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, *v as i32) ).collect();
                Cow::Owned( res )
            }
        }
    }    

    pub fn to_float(&self) -> Cow<VertexMap<f32>> {
        use VMapTypes::*;
        match &self.contents {
            VMFLOAT(vmap) => Cow::Borrowed(vmap),
            VMINT(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, *v as f32) ).collect();
                Cow::Owned( res )
            },
            VMBOOL(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, (*v as u32) as f32) ).collect();
                Cow::Owned( res )
            }
        }
    }    
    
    pub fn to_bool(&self) -> Cow<VertexMap<bool>> {
        use VMapTypes::*;
        match &self.contents {
            VMBOOL(vmap) => Cow::Borrowed(vmap),
            VMINT(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, v != &0) ).collect();
                Cow::Owned( res )
            },
            VMFLOAT(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, v != &0f32) ).collect();
                Cow::Owned( res )
            }
        }
    }      

    pub fn subset<'a, I>(&self,  vertices:I) -> Self where I: Iterator<Item=&'a Vertex> {
        use VMapTypes::*;        
        let selected:VertexSet = vertices.cloned().collect();
        match &self.contents {
            VMBOOL(vmap) => {
                let res = vmap.iter().filter(|&(k,_)| selected.contains(k)).map(|(k,v)| (*k,*v)).collect();
                PyVMap::new_bool(res)
            },
            VMINT(vmap) => {
                let res = vmap.iter().filter(|&(k,_)| selected.contains(k)).map(|(k,v)| (*k,*v)).collect();
                PyVMap::new_int(res)
            },
            VMFLOAT(vmap) => {
                let res = vmap.iter().filter(|&(k,_)| selected.contains(k)).map(|(k,v)| (*k,*v)).collect();
                PyVMap::new_float(res)                
            }
        }
    }
}

impl AttemptCast for PyVMap {
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

#[pymethods]
impl PyVMap {
    #[new]
    fn new_py(obj:&PyAny) -> PyResult<Self> {
        // Important: check bool FIRST as bools also coerce to int
        let val:PyResult<VertexMap<bool>> = obj.extract();
        if let Ok(map) = val {
            return Ok(PyVMap::new(VMapTypes::VMBOOL(map)))
        }
            
        let val:PyResult<VertexMap<i32>> = obj.extract();
        if let Ok(map) = val {
            return Ok(PyVMap::new(VMapTypes::VMINT(map)))
        }
    
        let val:PyResult<VertexMap<f32>> = obj.extract();
        if let Ok(map) = val {
            return Ok(PyVMap::new(VMapTypes::VMFLOAT(map)))
        }

        return Err(PyTypeError::new_err( format!("Cannot create map from {:?}", obj) ))
    }

    /// Sorts the vertices by their respective values, in ascending order.
    /// 
    /// If the map contains floats vertices with NaN values appear last.
    #[pyo3(text_signature="($self,/,reverse=False)")]    
    #[args(reverse=false)]
    pub fn rank(&self, reverse:bool) -> PyResult<Vec<u32>> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => {
                let mut vec:Vec<(u32,i32)>  = vmap.iter().map(|(k,v)| (*k,*v)).collect();
                if !reverse {
                    vec.sort_by(|(_,v1),(_,v2)| v1.cmp(v2));
                } else {
                    vec.sort_by(|(_,v1),(_,v2)| v2.cmp(v1));
                }
                vec.iter().map(|(k,_)| *k).collect()
            }
            VMFLOAT(vmap) => {
                // Floats cannot be ordered by default (because of NAN). The OrderedFloat wrapper
                // implements an ordered where NANs come last.
                let mut vec:Vec<(u32,OrderedFloat<f32>)>  = vmap.iter().map(|(k,v)| (*k,OrderedFloat(*v))).collect();
                vec.sort_by(|(_,v1),(_,v2)| v1.cmp(v2));
                vec.iter().map(|(k,_)| *k).collect()
            }
            VMBOOL(vmap) => {
                let mut vec:Vec<(u32,bool)>  = vmap.iter().map(|(k,v)| (*k,*v)).collect();
                if !reverse {
                    vec.sort_by(|(_,v1),(_,v2)| v1.cmp(v2));
                } else {
                    vec.sort_by(|(_,v1),(_,v2)| v2.cmp(v1));
                }
                vec.iter().map(|(k,_)| *k).collect()
            }
        };
        Ok(res)
    }

    /// Returns all vertices contained in the map.
    #[pyo3(text_signature="($self,/)")]    
    pub fn keys(&self) -> PyResult<Vec<u32>> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.keys().cloned().collect(),
            VMFLOAT(vmap) => vmap.keys().cloned().collect(),
            VMBOOL(vmap) => vmap.keys().cloned().collect(),
        };
        Ok(res)
    }

    /// Returns a selection of keys from the map depending on the type. 
    /// 
    /// For a boolean map it returns all vertices whose value are `True`. For all other maps,
    /// it simply returns all vertices.
    #[pyo3(text_signature="($self,/)")]    
    pub fn collect(&self) -> PyResult<Vec<u32>> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.keys().cloned().collect::<Vec<u32>>(),
            VMFLOAT(vmap) => vmap.keys().cloned().collect::<Vec<u32>>(),
            VMBOOL(vmap) => vmap.iter().filter_map(|(k,v)| if *v {Some(*k)} else {None} ).collect::<Vec<u32>>(),
        };
        Ok(res)
    }

    /// Returns all values that appear in this map.
    #[pyo3(text_signature="($self,/)")]
    pub fn values<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.values().cloned().collect::<Vec<i32>>().to_object(py),
            VMFLOAT(vmap) => vmap.values().cloned().collect::<Vec<f32>>().to_object(py),
            VMBOOL(vmap) => vmap.values().cloned().collect::<Vec<bool>>().to_object(py),
        };
        Ok(res)        
    }

    /// Returns the contents of this map as (vertex,value) pairs.
    #[pyo3(text_signature="($self,/)")]
    pub fn items<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.iter().map(|(k,v)| (*k, *v)).collect::<Vec<(u32, i32)>>().to_object(py),
            VMFLOAT(vmap) => vmap.iter().map(|(k,v)| (*k, *v)).collect::<Vec<(u32, f32)>>().to_object(py),
            VMBOOL(vmap) => vmap.iter().map(|(k,v)| (*k, *v)).collect::<Vec<(u32, bool)>>().to_object(py),
        };
        Ok(res)        
    }

    /// Returns the sum of all values in this map. Boolean values are converted
    /// to $\\{0,1\\}$ before the computation.
    #[pyo3(text_signature="($self,/)")]
    pub fn sum<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.values().sum::<i32>().to_object(py),
            VMFLOAT(vmap) => vmap.values().sum::<f32>().to_object(py),
            VMBOOL(vmap) => vmap.values().map(|v| *v as i32).sum::<i32>().to_object(py)
        };
        Ok(res)
    } 

    /// Returns the mean of all values in this map. Boolean values are converted
    /// to $\\{0,1\\}$ before the computation.
    #[pyo3(text_signature="($self,/)")]
    pub fn mean<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        use VMapTypes::*;
        let res:f32 = match &self.contents {
            VMINT(vmap) => vmap.values().sum::<i32>() as f32 / vmap.len() as f32,
            VMFLOAT(vmap) => vmap.values().sum::<f32>() / vmap.len() as f32,
            VMBOOL(vmap) => vmap.values().map(|v| *v as i32).sum::<i32>() as f32 / vmap.len() as f32
        };
        Ok(res.to_object(py))
    } 

    /// Returns the minimum of all values in this map. Uses the convention
    /// $\\min \\{\\text{True}, \\text{False}\\} = \\text{False}$
    #[pyo3(text_signature="($self,/)")]
    pub fn min<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.values().min().to_object(py),
            VMFLOAT(vmap) => vmap.values().reduce(|acc, v| if acc <= v {acc} else {v} ).to_object(py),
            VMBOOL(vmap) => vmap.values().reduce(|acc, v| if !acc {&false} else {v} ).to_object(py),
        };
        Ok(res)
    } 
     
    /// Returns the minimum of all values in this map. Uses the convention
    /// $\\max \\{\\text{True}, \\text{False}\\} = \\text{True}$
    #[pyo3(text_signature="($self,/)")]    
    pub fn max<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.values().max().to_object(py),
            VMFLOAT(vmap) => vmap.values().reduce(|acc, v| if acc >= v {acc} else {v} ).to_object(py),
            VMBOOL(vmap) => vmap.values().reduce(|acc, v| if *acc {&true} else {v} ).to_object(py),
        };
        Ok(res)
    } 

    /// Returns whether the map contains negative values. This method always returns `False`
    /// if the values are booleans.
    #[pyo3(text_signature="($self,/)")]       
    fn has_negative(&self) -> bool {
        use crate::VMapTypes::*;
        match &self.contents {
            VMINT(vmap) => vmap.values().any(|v| v < &0),
            VMFLOAT(vmap) => vmap.values().any(|v| v < &0f32),
            VMBOOL(_) => false
        }
    }

    /// Returns whether the map contains zero values. In the case of boolean values,
    /// returns whether there is at least one values that is `False`.
    #[pyo3(text_signature="($self,/)")]       
    fn has_zeros(&self) -> bool {
        use crate::VMapTypes::*;
        match &self.contents {
            VMINT(vmap) => vmap.values().any(|v| v == &0),
            VMFLOAT(vmap) => vmap.values().any(|v| v == &0f32),
            VMBOOL(vmap) => vmap.values().any(|v| !v)
        }
    }    

    /// Returns a boolean map which indicates which values in this map
    /// are `NaN`.
    #[pyo3(text_signature="($self,/)")]      
    fn is_nan(&self) -> PyResult<PyVMap> {
        use VMapTypes::*;
        let res = match &self.contents {
            VMINT(vmap) => vmap.iter().map(|(k,_)| (*k,false) ).collect(),
            VMFLOAT(vmap) => vmap.iter().map(|(k,v)| (*k, v.is_nan() ) ).collect(),
            VMBOOL(vmap) => vmap.iter().map(|(k,_)| (*k,false) ).collect(),
        };
        Ok(PyVMap::new_bool(res))
    }

    fn __len__(&self) -> usize {
        use crate::VMapTypes::*;
        match &self.contents {
            VMINT(vmap) => vmap.len(),
            VMFLOAT(vmap) => vmap.len(),
            VMBOOL(vmap) => vmap.len()    
        }
    }

    fn __contains__(&self, key: u32) -> bool {
        use VMapTypes::*;
        match &self.contents {
            VMINT(vmap) => vmap.contains_key(&key),
            VMFLOAT(vmap) => vmap.contains_key(&key),
            VMBOOL(vmap) => vmap.contains_key(&key)
        }
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn __repr__(&self) -> PyResult<String> {
        use VMapTypes::*;
        let mut res = String::new();
        match &self.contents {
            VMINT(vmap) => {
                res += &"VMap[int] {".to_string();
                let mut keys:Vec<_> = vmap.keys().collect();
                keys.sort();
                res += &join( keys.iter().map(|k| format!("{}: {}", k, vmap.get(k).unwrap()) ), ", ");
                res += "}";
            },
            VMFLOAT(vmap) => {
                res += &"VMap[float] {".to_string();
                let mut keys:Vec<_> = vmap.keys().collect();
                keys.sort();
                res += &join( keys.iter().map(|k| format!("{}: {}", k, vmap.get(k).unwrap()) ), ", ");
                res += "}";
            },
            VMBOOL(vmap) => {
                res += &"VMap[bool] {".to_string();
                let mut keys:Vec<_> = vmap.keys().collect();
                keys.sort();
                res += &join( keys.iter().map(|k| format!("{}: {}", k, vmap.get(k).unwrap()) ), ", ");
                res += "}";
            },
        }
        Ok(res)
    }

    fn __abs__(&self) -> PyResult<Self> {
        use VMapTypes::*;
        use super::ducktype::Ducktype::*;

        let res = match &self.contents {
            VMINT(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, v.abs() )).collect();
                VMINT(res)
            },
            VMFLOAT(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, v.abs() )).collect();
                VMFLOAT(res)
            },
            VMBOOL(vmap) => {
                let res = vmap.iter().map(|(k,v)| (*k, *v )).collect();
                VMBOOL(res)
            },
        };
        Ok(PyVMap::new(res))
    }

    fn __add__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        use VMapTypes::*;

        // Try to cast argument to VMap and combine entry-wise.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyVMap> {

            // If either of the arguments contains floats we need to downcast to that
            if self.is_float() || map.is_float() {
                let (left, right) = (self.to_float(), map.to_float());
                let res = combine(&left, &right, &0f32, &0f32, |l,r| l+r);
                return Ok(PyVMap::new_float(res));
            }

            // Otherwise we can simply cast to ints.
            let  (left, right) = (self.to_int(), map.to_int());
            let res = combine(&left, &right, &0, &0, |l,r| l+r);
            return Ok(PyVMap::new_int(res));
        });

        return_some!(res);

        // Try to cast argument to primite and apply to all entries
        use super::ducktype::Ducktype::*;
        match Ducktype::from(obj) {
            INT(r) => {
                if let VMFLOAT(vmap) = &self.contents {
                    let r = r as f32;
                    let res = map(&vmap, |l| l+r);
                    Ok( PyVMap::new_float(res) )
                } else {
                    let vmap = self.to_int();
                    let res = map(&vmap, |l| l+r);
                    Ok( PyVMap::new_int(res) )
                }
            },
            FLOAT(r) => {
                // Result will always be float
                let vmap = self.to_float();
                let res = map(&vmap, |l| l+r);
                Ok( PyVMap::new_float(res) )
            },
            BOOL(r) => {
                if let VMFLOAT(vmap) = &self.contents {
                    let r = if r { 1f32 } else { 0f32 };
                    let res = map(&vmap, |l| l+r);
                    Ok( PyVMap::new_float(res) )
                } else {
                    let vmap = self.to_int();
                    let r = if r { 1 } else { 0 };
                    let res = map(&vmap, |l| l+r);
                    Ok( PyVMap::new_int(res) )
                }
            },      
            x => Err(PyTypeError::new_err( format!("Addition with {:?} not supported", x) ))      
        }
    }

    fn __radd__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self.__add__(obj)
    }

    fn __sub__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self._sub(obj, false)
    }

    fn __rsub__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self._sub(obj, true)
    }

    fn _sub(&self, obj: &PyAny, reverse:bool) -> PyResult<PyVMap> {
        use VMapTypes::*;

        // Try to cast argument to VMap and combine entry-wise.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyVMap> {

            // If either of the arguments contains floats we need to downcast to that
            if self.is_float() || map.is_float() {
                let (left, right) = (self.to_float(), map.to_float());
                let res = if !reverse { combine(&left, &right, &0f32, &0f32, |l,r| l-r) }
                                    else { combine(&left, &right, &0f32, &0f32, |l,r| r-l) };
                return Ok(PyVMap::new_float(res));
            }

            // Otherwise we can simply cast to ints.
            let (left, right) = (self.to_int(), map.to_int());
            let res = if !reverse { combine(&left, &right, &0, &0, |l,r| l-r) }
                             else { combine(&left, &right, &0, &0, |l,r| r-l) };
            return Ok(PyVMap::new_int(res));
        });

        return_some!(res);

        // Try to cast argument to primite and apply to all entries
        use super::ducktype::Ducktype::*;
        match Ducktype::from(obj) {
            INT(r) => {
                if let VMFLOAT(vmap) = &self.contents {
                    let r = r as f32;
                    let res = if !reverse { map(&vmap, |l| l-r) }
                                     else { map(&vmap, |l| r-l) };
                    Ok( PyVMap::new_float(res) )
                } else {
                    let vmap = self.to_int();
                    let res = if !reverse { map(&vmap, |l| l-r) }
                                     else { map(&vmap, |l| r-l) };
                    Ok( PyVMap::new_int(res) )
                }
            },
            FLOAT(r) => {
                // Result will always be float
                let vmap = self.to_float();
                let res = if !reverse { map(&vmap, |l| l-r) }
                                 else { map(&vmap, |l| r-l) };
                Ok( PyVMap::new_float(res) )
            },
            BOOL(r) => {
                if let VMFLOAT(vmap) = &self.contents {
                    let r = if r { 1f32} else { 0f32 };
                    let res = if !reverse { map(&vmap, |l| l-r) }
                                     else { map(&vmap, |l| r-l) };
                    Ok( PyVMap::new_float(res) )
                } else {
                    let vmap = self.to_int();
                    let r = if r { 1 } else { 0 };
                    let res = if !reverse { map(&vmap, |l| l-r) }
                                     else { map(&vmap, |l| r-l) };
                    Ok( PyVMap::new_int(res) )
                }
            },   
            x => Err(PyTypeError::new_err( format!("Subtraction with {:?} not supported", x) ))         
        }
    }

    fn __mul__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        use VMapTypes::*;

        // Try to cast argument to VMap and combine entry-wise.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyVMap> {

            // If either of the arguments contains floats we need to downcast to that
            if self.is_float() || map.is_float() {
                let (left, right) = (self.to_float(), map.to_float());
                let res = combine(&left, &right, &1f32, &1f32, |l,r| l*r);
                return Ok(PyVMap::new_float(res));
            }

            // Otherwise we can simply cast to ints.
            let  (left, right) = (self.to_int(), map.to_int());
            let res = combine(&left, &right, &1, &1, |l,r| l*r);
            return Ok(PyVMap::new_int(res));
        });

        return_some!(res);

        // Try to cast argument to primite and apply to all entries
        use super::ducktype::Ducktype::*;
        match Ducktype::from(obj) {
            INT(r) => {
                if let VMFLOAT(vmap) = &self.contents {
                    let r = r as f32;
                    let res = map(&vmap, |l| l*r);
                    Ok( PyVMap::new_float(res) )
                } else {
                    let vmap = self.to_int();
                    let res = map(&vmap, |l| l*r);
                    Ok( PyVMap::new_int(res) )
                }
            },
            FLOAT(r) => {
                // Result will always be float
                let vmap = self.to_float();
                let res = map(&vmap, |l| l*r);
                Ok( PyVMap::new_float(res) )
            },
            BOOL(r) => {
                if let VMFLOAT(vmap) = &self.contents {
                    let r = if r { 1f32 } else { 0f32 };
                    let res = map(&vmap, |l| l*r);
                    Ok( PyVMap::new_float(res) )
                } else {
                    let vmap = self.to_int();
                    let r = if r { 1 } else { 0 };
                    let res = map(&vmap, |l| l*r);
                    Ok( PyVMap::new_int(res) )
                }
            },      
            x => Err(PyTypeError::new_err( format!("Addition with {:?} not supported", x) ))      
        }
    }     
    
    fn __rmul__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self.__mul__(obj)
    }    

    fn __truediv__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self._truediv(obj, false)
    }

    fn __rtruediv__(&self,obj: &PyAny) -> PyResult<PyVMap> {
        self._truediv(obj, true)
    }    

    fn _truediv(&self, obj: &PyAny, reverse: bool) -> PyResult<PyVMap> {        
        use VMapTypes::*;

        // Try to cast argument to VMap and combine entry-wise.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyVMap> {
            // The result of a true division is _always_ float.
            let (left, right) = (self.to_float(), map.to_float());
            let res = if !reverse { combine(&left, &right, &1f32, &1f32, |l,r| l/r) }
                             else { combine(&left, &right, &1f32, &1f32, |l,r| r/l) };
            return Ok(PyVMap::new_float(res));
        });

        return_some!(res);

        // Try to cast argument to primite and apply to all entries
        use super::ducktype::Ducktype::*;
        let r = match Ducktype::from(obj) {
            INT(r) => (r as f32),
            FLOAT(r) => r,
            BOOL(r) => if r { 1f32 } else { 0f32 },
            x => return Err(PyTypeError::new_err( format!("Addition with {:?} not supported", x) ))      
        };
        let vmap = self.to_float();
        let res = if !reverse { map(&vmap, |l| l/r) }
                         else { map(&vmap, |l| r/l) };
        Ok( PyVMap::new_float(res) )        
    }   

    fn __floordiv__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self._floordiv(obj, false)
    }

    fn __rfloordiv__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self._floordiv(obj, true)
    }

    fn _floordiv(&self, obj: &PyAny, reverse: bool) -> PyResult<PyVMap> {
        // Integer division (`//` in python). We allow this operation for
        // floats as well in which case we first compute the float division and 
        // then cast to int. 
        // Note: The floor division really applies floor, meaning that if the result
        //       is negative, we round *away* from zero. This is consistent with how
        //       Python handles this operator.
        use VMapTypes::*;

        // Try to cast argument to VMap and combine entry-wise.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyVMap> {

            // If either of the arguments contains floats we have to apply
            // floating point operations anyway and then round don.
            let res = if self.is_float() || map.is_float() {
                let (left, right) = (self.to_float(), map.to_float());
                if !reverse {
                    combine(&left, &right, &1f32, &1f32, |l,r| f32::floor(l/r) as i32 )
                } else {
                    combine(&left, &right, &1f32, &1f32, |l,r| f32::floor(r/l) as i32)
                }
            } else {
                let  (left, right) = (self.to_int(), map.to_int());
                if !reverse {
                    combine(&left, &right, &1, &1, |l,r| floor_div(l,r))
                } else {
                    combine(&left, &right, &1, &1, |l,r| floor_div(r,l))
                }
            };

            // Otherwise we can simply cast to ints.
            return Ok(PyVMap::new_int(res));
        });

        return_some!(res);

        // Try to cast argument to primite and apply to all entries
        use super::ducktype::Ducktype::*;

        // Cast bool to int to avoid some code dubplication below
        let r = match Ducktype::from(obj) {
            INT(r) => INT(r),
            FLOAT(r) => FLOAT(r),
            BOOL(r) => INT(r as i32),
            x => return Err(PyTypeError::new_err( format!("Division with {:?} not supported", x) ))      
        };

        match r {
            INT(r) => {
                let res = if let VMFLOAT(vmap) = &self.contents {
                    let r = r as f32;
                    if !reverse {
                        map(&vmap, |l| (l/r) as i32 )
                    } else {
                        map(&vmap, |l| (r/l) as i32 )
                    }
                } else {
                    let vmap = self.to_int();
                    if !reverse {
                        map(&vmap, |l| floor_div(l,&r))
                    } else {
                        map(&vmap, |l| floor_div(&r,l))
                    }
                };
                Ok( PyVMap::new_int(res) )
            },
            FLOAT(r) => {
                let vmap = self.to_float();
                let res = if !reverse {
                    map(&vmap, |l| (l/r) as i32 )
                } else {
                    map(&vmap, |l| (r/l) as i32 )
                };
                Ok(PyVMap::new_int(res))
            },
            x => Err(PyTypeError::new_err( format!("Division with {:?} not supported", x) ))      
        }
    }

    fn __mod__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self._mod(obj, false)
    }

    fn __rmod__(&self, obj: &PyAny) -> PyResult<PyVMap> {
        self._mod(obj, true)
    }
    
    fn _mod(&self, obj: &PyAny, reverse: bool) -> PyResult<PyVMap> {
        use super::ducktype::Ducktype::*;
        use VMapTypes::*;

        let res = PyVMap::try_cast(obj, |map| {
            if self.is_float() || map.is_float() {
                let (left, right) = (self.to_float(), map.to_float());
                let res = if !reverse {
                    combine(&left, &right, &1f32, &1f32, |l,r| l % r )
                } else {
                    combine(&left, &right, &1f32, &1f32, |l,r| r % l )
                };
                Ok(PyVMap::new_float(res))
            } else {
                let  (left, right) = (self.to_int(), map.to_int());
                let res = if !reverse {
                    combine(&left, &right, &1, &1, |l,r| l % r)
                } else {
                    combine(&left, &right, &1, &1, |l,r| l % r)
                };
                Ok(PyVMap::new_int(res))
            }
        });

        return_some!(res);

        match Ducktype::from(obj) {
            INT(r) => {
                if let VMFLOAT(vmap) = &self.contents {
                    let r = r as f32;
                    let res = if !reverse { map(&vmap, |l| l%r ) }
                                     else { map(&vmap, |l| r%l ) };
                    Ok(PyVMap::new_float(res))
                } else {
                    let vmap = self.to_int();
                    let res = if !reverse { map(&vmap, |l| l%r ) }
                                     else { map(&vmap, |l| r%l ) };
                    Ok(PyVMap::new_int(res))                    
                }
            },
            FLOAT(r) => {
                let vmap = self.to_float();
                let res = if !reverse { map(&vmap, |l| l%r ) }
                                    else { map(&vmap, |l| r%l ) };
                Ok(PyVMap::new_float(res))
            },            
            x => Err(PyTypeError::new_err( format!("Modulo with {:?} not supported", x) ))
        }
    }    

    fn __pow__(&self, obj: &PyAny, _modulo: Option<i32>) -> PyResult<PyVMap> {
        self._pow(obj, false)
    }

    fn __rpow__(&self, obj: &PyAny, _modulo: Option<i32>) -> PyResult<PyVMap> { 
        self._pow(obj, true)
    }

    fn _pow(&self, obj: &PyAny, reverse: bool) -> PyResult<PyVMap> {
        // Note: The argument 'm' is the option modulo argument for the python
        // __pow__ method. We do not currently use it here.
        use super::ducktype::Ducktype::*;
        use VMapTypes::*;

            // Try to cast argument to VMap and combine entry-wise.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyVMap> {

            // If either of the arguments contains floats or if the RHS 
            // operator contains negative values we have to cast to float.
            if self.is_float() || map.is_float() 
                || (!reverse && map.has_negative()) 
                || (reverse && self.has_negative()) {
                let (left, right) = (self.to_float(), map.to_float());
                let res = if !reverse {
                    combine(&left, &right, &0f32, &0f32, |l,r| l.powf(*r) )
                } else {
                    combine(&left, &right, &0f32, &0f32, |l,r| r.powf(*l) )
                };
                return Ok(PyVMap::new_float(res));
            }

            // Otherwise we can simply cast to ints.
            let  (left, right) = (self.to_int(), map.to_int());
            let res = if !reverse { 
                combine(&left, &right, &0, &0, |l,r| l.pow(*r as u32))
            } else {
                combine(&left, &right, &0, &0, |l,r| r.pow(*l as u32))
            };
            return Ok(PyVMap::new_int(res));
        });

        return_some!(res);           

        let r = match Ducktype::from(obj) {
            INT(r) => INT(r),
            FLOAT(r) => FLOAT(r),
            BOOL(r) => INT(r as i32),
            x => return Err(PyTypeError::new_err( format!("Exponentiation with {:?} not supported", x) ))
        };

        match r {
            INT(r) => {
                if (reverse && self.has_negative()) || (!reverse && r < 0) {
                    let vmap = self.to_float();
                    let r = r as f32;
                    let res = if !reverse {
                        map(&vmap, |l| l.powf(r) )
                    } else {
                        map(&vmap, |l| r.powf(*l) )
                    };
                    Ok(PyVMap::new_float(res))
                } else {
                    let r = r as u32;
                    let vmap = self.to_int();
                    let res = map(&vmap, |l| l.pow(r));
                    Ok(PyVMap::new_int(res))
                }
            },
            FLOAT(r) => {
                let vmap = self.to_float();
                let res = if !reverse {
                    map(&vmap, |l| l.powf(r) )
                } else {
                    map(&vmap, |l| r.powf(*l) )
                };
                Ok(PyVMap::new_float(res))
            },
            x => Err(PyTypeError::new_err( format!("Exponentiation with {:?} not supported", x) ))
        }
    }     

    fn __delitem__(&mut self, key: u32) -> PyResult<()> {
        use VMapTypes::*;

        match &mut self.contents {
            VMINT(vmap) => {vmap.remove(&key);},
            VMFLOAT(vmap) => {vmap.remove(&key);},
            VMBOOL(vmap) => {vmap.remove(&key);},
        };
        Ok(())
    }
    
    fn __setitem__(&mut self, key: u32, val: &PyAny) -> PyResult<()> {
        use super::ducktype::Ducktype::*;
        use VMapTypes::*;

        let val = Ducktype::from(val);

        // If this map contains floats we can simply cast `val` to float
        // and insert it.
        if let VMFLOAT(vmap) = &mut self.contents {
            vmap.insert(key, val.into());
            return Ok(())
        }

        if let VMINT(vmap) = &mut self.contents {
            if let FLOAT(val) = val {
                let mut res = self.to_float().into_owned();
                res.insert(key, val);
                self.contents = VMFLOAT(res);
            } else {
                vmap.insert(key, val.into());
            }
            return Ok(())            
        }

        if let VMBOOL(vmap) = &mut self.contents {
            if let FLOAT(val) = val {
                let mut res = self.to_float().into_owned();
                res.insert(key, val);
                self.contents = VMFLOAT(res);
            } else if let INT(val) = val {
                let mut res = self.to_int().into_owned();
                res.insert(key, val);
                self.contents = VMINT(res);
            } else {
                vmap.insert(key, val.into());
            }
            return Ok(())            
        }        

        Ok(())
    }

    fn __getitem__<'py>(&self, py: Python<'py>, obj: &PyAny) -> PyResult<PyObject> {
        use super::ducktype::Ducktype::*;
        use VMapTypes::*;

        // Attempt to cast to PyMapBool. If successful, we use the other map
        // as a boolean index.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyObject> {
            if let VMBOOL(vmap) = &map.contents {
                let res = match &self.contents {
                    VMINT(vmap_self) => 
                        VMINT(vmap_self.iter()
                            .filter(|(k,_)| *vmap.get(k).unwrap_or(&false))
                            .map(|(k,v)| (*k, *v))
                            .collect()
                        ),
                    VMFLOAT(vmap_self) =>                         
                        VMFLOAT(vmap_self.iter()
                            .filter(|(k,_)| *vmap.get(k).unwrap_or(&false))
                            .map(|(k,v)| (*k, *v))
                            .collect()
                        ),
                    VMBOOL(vmap_self) =>
                        VMBOOL(vmap_self.iter()
                            .filter(|(k,_)| *vmap.get(k).unwrap_or(&false))
                            .map(|(k,v)| (*k, *v))
                            .collect()
                        )
                };
                let temp = Py::new(py, PyVMap::new(res))?;
                Ok(temp.to_object(py))            
            } else {
                Err(PyValueError::new_err(""))
            }
        });

        return_some!(res);

        // Attempt to cast to list 
        let keys_maybe = obj.extract::<Vec<u32>>();
        if let PyResult::Ok(keys) = keys_maybe {
            let res = self.subset(keys.iter());
            let temp = Py::new(py, res)?;
            return Ok(temp.to_object(py));
        }

        // Attempt to cast to set
        let keys_maybe = obj.extract::<std::collections::HashSet<u32> >();
        if let PyResult::Ok(keys) = keys_maybe {
            let res = self.subset(keys.iter());
            let temp = Py::new(py, res)?;
            return Ok(temp.to_object(py));
        }


        // Attempt to cast to single primitives
        match Ducktype::from(obj) {
            INT(x) => {
                let res = if x < 0 { None } else {
                    match &self.contents {
                        VMINT(vmap) => vmap.get(&(x as u32)).map(|v| v.to_object(py)),
                        VMFLOAT(vmap) => vmap.get(&(x as u32)).map(|v| v.to_object(py)),
                        VMBOOL(vmap) => vmap.get(&(x as u32)).map(|v| v.to_object(py))
                    }   
                };
                if let Some(res) = res {
                    return Ok(res);
                } else {
                    return Err(PyValueError::new_err( format!("Invalid index {:?}", x) ));
                }
            }
            _ => return Err(PyTypeError::new_err( format!("Unsupported index {:?}", obj) ))
        }
    }

    fn __richcmp__(&self, obj: &PyAny, op: CompareOp) -> PyResult<PyVMap> {
        use super::ducktype::Ducktype::*;
        use VMapTypes::*;

        // Attempt to cast argument to PyVMap. If this succeedds we compare all 
        // elements of the two maps. Missing keys are handled as follows:
        //    - If `self` contains a key k which `other` does not contain, the 
        //      result for k will be `True`. This is because we can easily exclude
        //      these keys if we want to (by restricting to the keys of `other` afterwards)
        //    - If `other` contains a key k which `self` does not contain, it is ignored.
        let res = PyVMap::try_cast(obj, |map| -> PyResult<PyVMap> {
            if self.is_float() || map.is_float() {
                let (vmap, vmap_other) = (self.to_float(), map.to_float());
                let res:VertexMap<bool> = match op {
                    CompareOp::Lt => combine(&vmap, &vmap_other, &f32::INFINITY, &f32::INFINITY, |v_1,v_2| v_1 <  v_2 ),
                    CompareOp::Le => combine(&vmap, &vmap_other, &f32::INFINITY, &f32::INFINITY, |v_1,v_2| v_1 <= v_2 ),
                    CompareOp::Eq => {
                        // We use NAN as defaults for equality because NAN != NAN
                        let mut res = combine(&vmap, &vmap_other, &f32::NAN, &f32::NAN, |v_1,v_2| v_1 == v_2 );
                        // `res` now contains all keys k common to `self` and `other` whose values agree.
                        // We are missing the keys k in `self` which are not present in `other`

                        res.extend( vmap.keys().filter(|k| !vmap_other.contains_key(k)).map(|k| (*k, true)) );
                        res
                    },
                    CompareOp::Ne => {
                        let mut res = combine(&vmap, &vmap_other, &f32::NAN, &f32::NAN, |v_1, v_2| v_1 != v_2 );
                        // `res` now contains all keys k common to `self` and `other` whose values disagree,
                        // as well as all keys from `self` for which `other` has no key since v_1 != NAN 
                        // is always true, even for v_1 = NAN.
                        // `res` further contains all keys from `other` which are not in `self` and we 
                        // need to remove them.
                        for k in vmap_other.keys() {
                            if !vmap.contains_key(k) {
                                res.remove(k);
                            }
                        }
                        res
                    },
                    CompareOp::Ge => combine(&vmap, &vmap_other, &f32::NEG_INFINITY, &f32::NEG_INFINITY, |v_1,v_2| v_1 >= v_2 ),
                    CompareOp::Gt => combine(&vmap, &vmap_other, &f32::NEG_INFINITY, &f32::NEG_INFINITY, |v_1,v_2| v_1 >  v_2 ),
                };
                
                return Ok(PyVMap::new_bool(res))
            }

            unimplemented!();
        });

        return_some!(res);

        let mut val = Ducktype::from(obj);

        // Otherwise we try to cast to a basic type
        match &self.contents {
            VMINT(vmap) => {
                // Cast bool to int so we don't have to handle it seperately
                if let BOOL(x) = val {
                    val = INT(x as i32);
                }

                let res = match val {
                    INT(val) => 
                        map_boxed(vmap, comparator::<i32,i32>(op, &val, &|v| v)),
                    FLOAT(val) =>
                        map_boxed(vmap, comparator::<i32,f32>(op, &val, &|v| v as f32)),
                    x => return Err(PyTypeError::new_err( format!("Comparison operation with {:?} not supported", x) ))
                };
                return Ok(PyVMap::new_bool(res));
            },
            VMFLOAT(vmap) => {
                let val:f32 = val.into(); 
                let res = map_boxed(vmap, comparator::<f32,f32>(op, &val, &|v| v));
                return Ok(PyVMap::new_bool(res));
            },
            VMBOOL(vmap) => {
                let res = match val {
                    BOOL(val) => 
                        map_boxed(vmap, comparator::<bool,bool>(op, &val, &|v| v)),
                    INT(val) => 
                        map_boxed(vmap, comparator::<bool,i32>(op, &val, &|v| v as i32)),
                    FLOAT(val) => 
                        map_boxed(vmap, comparator::<bool,f32>(op, &val, &|v| (v as i32) as f32)),
                    x => return Err(PyTypeError::new_err( format!("Comparison operation with {:?} not supported", x) ))
                };
                return Ok(PyVMap::new_bool(res));                
            }
        }
    }    

    fn __invert__(&self) -> PyResult<PyVMap> {
        let vmap = self.to_bool();
        let res:VertexMap<bool> = map(&vmap, |x| !x);
        Ok(PyVMap::new_bool(res))
    }

    fn __or__(&self, other: &PyVMap) -> PyResult<PyVMap> {
        let (left, right) = (self.to_bool(), other.to_bool());
        let res = combine(&left, &right, &true, &true, |l,r| l | r);
        Ok(PyVMap::new_bool(res))
    }   
    
    fn __and__(&self, other: &PyVMap) -> PyResult<PyVMap> {
        let (left, right) = (self.to_bool(), other.to_bool());
        let res = combine(&left, &right, &true, &true, |l,r| l & r);
        Ok(PyVMap::new_bool(res))
    }    
    
    fn __xor__(&self, other: &PyVMap) -> PyResult<PyVMap> {
        let (left, right) = (self.to_bool(), other.to_bool());
        let res = combine(&left, &right, &true, &true, |l,r| l == r);
        Ok(PyVMap::new_bool(res))
    }     
}