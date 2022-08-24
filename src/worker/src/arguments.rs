use wasm_bindgen::JsValue;
use crate::MessageReturn;


pub trait Arguments {
    fn from_vec(v: Vec<JsValue>) -> Self;
}

impl<> Arguments for () {
    fn from_vec(v: Vec<JsValue>) -> Self {
        match &v[..] {
            [] => (),
            _ => unreachable!(),
        }
    }
}

impl<A> Arguments for (A,) where A: MessageReturn {
    fn from_vec(v: Vec<JsValue>) -> Self {
        match &v[..] {
            [a] => (A::from_js(a),),
            _ => unreachable!(),
        }
    }
}

impl<A, B> Arguments for (A, B)
    where A: MessageReturn,
          B: MessageReturn {
    fn from_vec(v: Vec<JsValue>) -> Self {
        match &v[..] {
            [a, b] => (
                A::from_js(a),
                B::from_js(b),
            ),
            _ => unreachable!(),
        }
    }
}

impl<A, B, C> Arguments for (A, B, C)
    where A: MessageReturn,
          B: MessageReturn,
          C: MessageReturn {
    fn from_vec(v: Vec<JsValue>) -> Self {
        match &v[..] {
            [a, b, c] => (
                A::from_js(a),
                B::from_js(b),
                C::from_js(c),
            ),
            _ => unreachable!(),
        }
    }
}

impl<A, B, C, D> Arguments for (A, B, C, D)
    where A: MessageReturn,
          B: MessageReturn,
          C: MessageReturn,
          D: MessageReturn {
    fn from_vec(v: Vec<JsValue>) -> Self {
        match &v[..] {
            [a, b, c, d] => (
                A::from_js(a),
                B::from_js(b),
                C::from_js(c),
                D::from_js(d),
            ),
            _ => unreachable!(),
        }
    }
}

impl<A, B, C, D, E> Arguments for (A, B, C, D, E)
    where A: MessageReturn,
          B: MessageReturn,
          C: MessageReturn,
          D: MessageReturn,
          E: MessageReturn {
    fn from_vec(v: Vec<JsValue>) -> Self {
        match &v[..] {
            [a, b, c, d, e] => (
                A::from_js(a),
                B::from_js(b),
                C::from_js(c),
                D::from_js(d),
                E::from_js(e),
            ),
            _ => unreachable!(),
        }
    }
}
