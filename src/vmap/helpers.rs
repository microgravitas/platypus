use std::collections::HashMap;

use pyo3::pyclass::CompareOp;

use super::VertexMap;


/// Simulates the Python floor division between to integers since 
/// Rust's behaviour differs for negative values. 
pub(crate) fn floor_div(a:&i32, b:&i32) -> i32 {
    let (res, md) = (a/b, a % b);
    
    if md == 0 || res > 0 {
        res
    } else {
        res-1
    }
}

pub(crate) fn pow_mod(a:i32, mut b:u32, m:i32) -> i32{
    let mut x = 1i64;
    let mut y = a as i64; 
    let m = m as i64;
    while b > 0 {
        if b%2 == 1 {
            x = (x*y) % m;
        }
        y = (y*y) % m; 
        b /= 2;
    }
    return (x % m) as i32;
}


/// Combines to vertex maps with types <T> and <U> using a combinator
/// function f(&T,&U) -> R and two default values to replace missing
pub fn combine<T, U, R>(left: &VertexMap<T>, right: &VertexMap<U>, 
                        ldef: &T, rdef: &U,
                        f: impl Fn(&T, &U) -> R) -> VertexMap<R> 
        where T: Copy, U: Copy
{
    let joint:VertexMap<(T,U)> = left.iter()
    .filter(|(k,_)| right.contains_key(k))
    .map(|(k,v)| (*k, ( *v , *right.get(k).unwrap() )))
    .collect();

    // Compute values of joint elements
    let mut res:VertexMap<R> = joint.iter()
        .map(|(k,(v1,v2))| (*k, f(v1, v2) ) )
        .collect();

    // Compute values of elements not contained in other
    let it = left.iter()
        .filter(|(k,_)| !joint.contains_key(k))
        .map(|(k,v)| (*k, f( v, rdef )));
    res.extend( it );

    // Compute values of elements only contained in other
    let it = right.iter()
        .filter(|(k,_)| !joint.contains_key(k))
        .map(|(k,v)| (*k, f( ldef, v )));
    res.extend( it );

    res
}

pub fn map<T,R>(vmap: &VertexMap<T>, f: impl Fn(&T) -> R) -> VertexMap<R> {
    vmap.iter().map(|(k,v)| (*k, f(v))).collect()
}

pub fn map_boxed<'a,T,R>(vmap: &VertexMap<T>, f: Box<dyn Fn(&T) -> R + 'a>) -> VertexMap<R> {
    vmap.iter().map(|(k,v)| (*k, f(v))).collect()
}

pub fn map_mut<T,R>(vmap: &mut VertexMap<T>, f: impl Fn(&T) -> R) -> VertexMap<R> {
    vmap.drain().map(|(k,v)| (k, f(&v))).collect()
}  

pub fn comparator<'a, U, T>(op: CompareOp, rhs:&'a T, cast: &'a dyn Fn(U) -> T) -> Box<dyn Fn(&U) -> bool + 'a>
    where T: std::cmp::PartialOrd + Sized + Copy,
          U: Copy + 'a{
    let rhs:T = *rhs;
    match op {
        CompareOp::Lt => Box::new(move |v:&U| cast(*v) <  rhs),
        CompareOp::Le => Box::new(move |v:&U| cast(*v) <= rhs),
        CompareOp::Eq => Box::new(move |v:&U| cast(*v) == rhs),
        CompareOp::Ne => Box::new(move |v:&U| cast(*v) != rhs),
        CompareOp::Gt => Box::new(move |v:&U| cast(*v) >  rhs),
        CompareOp::Ge => Box::new(move |v:&U| cast(*v) >= rhs)
    }
}