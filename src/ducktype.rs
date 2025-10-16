use std::collections::HashSet;

use pyo3::{exceptions::PyTypeError, types::PyAnyMethods, Bound, PyAny, PyResult};
use graphbench::graph::Vertex;

//
// Helper methods
//
pub(crate) fn to_vertex_list(obj:&Bound<'_,PyAny>) -> PyResult<Vec<u32>>  {
    let cast:Result<Vec<Vertex>, _> = obj.extract();
    if let Ok(vec) = cast {
        return Ok(vec);
    }

    let cast:Result<HashSet<Vertex>, _> = obj.extract();
    if let Ok(set) = cast {
        return Ok(set.into_iter().collect());
    }

    Err(PyTypeError::new_err( format!("Cannot cast {:?} to a vertex list", obj) ))
}

//
// Limited duck types via enumeration
//
#[derive(Debug)]
pub enum Ducktype {
    INT(i32),
    FLOAT(f32),
    BOOL(bool),
    STRING(String),
    UNKNOWN
}

impl Ducktype {
    pub fn from(obj:&Bound<'_,PyAny>) ->  Self {
        if let Ok(x) = obj.extract::<bool>() {
            return Ducktype::BOOL(x)
        }
        if let Ok(x) = obj.extract::<i32>() {
            return Ducktype::INT(x)
        }
        if let Ok(x) = obj.extract::<f32>() {
            return Ducktype::FLOAT(x)
        }
        if let Ok(x) = obj.extract::<String>() {
            return Ducktype::STRING(x)
        }

        Ducktype::UNKNOWN
    }
}

impl Into<f32> for Ducktype {
    fn into(self) -> f32 {
        use Ducktype::*;
        match self {
            INT(v) => v as f32,
            FLOAT(v) => v,
            BOOL(v) => (v as i32) as f32,
            _ => panic!(),
        }
    }
}

impl Into<i32> for Ducktype {
    fn into(self) -> i32 {
        use Ducktype::*;
        match self {
            INT(v) => v ,
            BOOL(v) => v as i32,
            _ => panic!(),
        }
    }
}

impl Into<bool> for Ducktype {
    fn into(self) -> bool {
        use Ducktype::*;
        match self {
            BOOL(v) => v,
            _ => panic!(),
        }
    }
}


#[macro_export]
macro_rules! return_some{
    ($a:ident) => {
        if let Some(obj) = $a {
            return obj;
        }
    }
}
