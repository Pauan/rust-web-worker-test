use std::borrow::Borrow;
use wasm_bindgen::{JsValue, JsCast, UnwrapThrowExt};
use js_sys::{Uint8Array, Array};


pub trait MessageSend {
    type Output: MessageReturn;

    fn into_js<A>(value: A, transfer_list: &Array) -> JsValue where A: Borrow<Self>;
}

pub trait MessageReturn {
    fn from_js(value: &JsValue) -> Self;
}


impl MessageSend for JsValue {
    type Output = JsValue;

    fn into_js<A>(value: A, _transfer_list: &Array) -> JsValue where A: Borrow<Self> {
        value.borrow().clone()
    }
}

impl MessageReturn for JsValue {
    fn from_js(value: &JsValue) -> Self {
        value.clone()
    }
}


impl MessageSend for Vec<u8> {
    type Output = Vec<u8>;

    fn into_js<A>(value: A, transfer_list: &Array) -> JsValue where A: Borrow<Self> {
        let value = Uint8Array::from(value.borrow().as_slice());
        transfer_list.push(&value.buffer());
        value.into()
    }
}

impl MessageReturn for Vec<u8> {
    fn from_js(value: &JsValue) -> Self {
        let value: &Uint8Array = value.unchecked_ref();
        value.to_vec()
    }
}


impl MessageSend for String {
    type Output = String;

    fn into_js<A>(value: A, _transfer_list: &Array) -> JsValue where A: Borrow<Self> {
        JsValue::from(value.borrow())
    }
}

impl MessageReturn for String {
    fn from_js(value: &JsValue) -> Self {
        value.as_string().unwrap_throw()
    }
}


macro_rules! impl_number {
    ($($t:ty),*) => {
        $(impl MessageSend for $t {
            type Output = $t;

            fn into_js<A>(value: A, _transfer_list: &Array) -> JsValue where A: Borrow<Self> {
                JsValue::from(*value.borrow())
            }
        }

        impl MessageReturn for $t {
            fn from_js(value: &JsValue) -> Self {
                let value = value.as_f64().unwrap_throw();
                value as $t
            }
        })*
    };
}

impl_number!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
