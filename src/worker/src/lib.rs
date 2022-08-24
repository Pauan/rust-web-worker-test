use std::future::Future;
use std::task::{Poll, Context};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use futures_channel::mpsc::{UnboundedReceiver, unbounded};
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use gloo_events::EventListener;
use js_sys::{Array, Object, global};
use web_sys::{MessageEvent, DedicatedWorkerGlobalScope, WorkerOptions, WorkerType};
use crate::arguments::Arguments;

mod transfer;
mod arguments;
mod macros;

pub use transfer::{MessageSend, MessageReturn};


#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone)]
    type JsMessage;


    #[wasm_bindgen(method, getter)]
    fn loaded(this: &JsMessage) -> Option<bool>;

    #[wasm_bindgen(method, setter)]
    fn set_loaded(this: &JsMessage, value: bool);


    #[wasm_bindgen(method, getter)]
    fn call(this: &JsMessage) -> String;

    #[wasm_bindgen(method, setter)]
    fn set_call(this: &JsMessage, value: &str);


    #[wasm_bindgen(method, getter, js_name = value)]
    fn args(this: &JsMessage) -> Vec<JsValue>;

    #[wasm_bindgen(method, getter)]
    fn value(this: &JsMessage) -> JsValue;

    #[wasm_bindgen(method, setter)]
    fn set_value(this: &JsMessage, value: &JsValue);


    #[wasm_bindgen(method, getter)]
    fn id(this: &JsMessage) -> usize;

    #[wasm_bindgen(method, setter)]
    fn set_id(this: &JsMessage, value: usize);
}


#[derive(Debug)]
pub struct WorkerCall<'a> {
    worker: &'a Worker,
    name: &'static str,
    transfer: Array,
    args: Array,
}

impl<'a> WorkerCall<'a> {
    fn new(worker: &'a Worker, name: &'static str) -> Self {
        Self {
            worker,
            name,
            transfer: Array::new(),
            args: Array::new(),
        }
    }

    pub fn arg<A>(self, value: A) -> Self where A: MessageSend {
        self.args.push(&A::into_js(value, &self.transfer));
        self
    }

    pub fn send<A>(self) -> impl Future<Output = A> where A: MessageReturn {
        static ID: AtomicUsize = AtomicUsize::new(0);

        let id = ID.fetch_add(1, Ordering::SeqCst);

        let name = self.name;

        let message: JsMessage = Object::new().unchecked_into();
        message.set_call(name);
        message.set_id(id);
        message.set_value(&self.args);

        self.worker.send_message(&message, &self.transfer);

        // TODO use a single message disatcher instead
        let mut messages = self.worker.messages();

        async move {
            while let Some(message) = messages.next().await {
                if message.id() == id {
                    return A::from_js(&message.value());
                }
            }

            panic!("Message call {} failed: stream closed", name);
        }
    }
}


#[derive(Debug)]
struct WorkerMessages {
    receiver: UnboundedReceiver<JsMessage>,
    _listener: EventListener,
}

impl Stream for WorkerMessages {
    type Item = JsMessage;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}


#[derive(Debug)]
pub struct Worker {
    worker: web_sys::Worker,
}

impl Worker {
    pub fn new(url: &str) -> Self {
        Self {
            worker: web_sys::Worker::new_with_options(url, &WorkerOptions::new().type_(WorkerType::Module)).unwrap_throw(),
        }
    }

    fn messages(&self) -> WorkerMessages {
        let (sender, receiver) = unbounded();

        WorkerMessages {
            _listener: EventListener::new(&self.worker, "message", move |e| {
                let e: &MessageEvent = e.unchecked_ref();
                sender.unbounded_send(e.data().unchecked_into()).unwrap_throw();
            }),
            receiver,
        }
    }

    fn send_message(&self, message: &JsValue, transfer: &Array) {
        self.worker.post_message_with_transfer(message, transfer).unwrap_throw()
    }

    pub fn call(&self, name: &'static str) -> WorkerCall<'_> {
        WorkerCall::new(self, name)
    }

    pub fn wait_loaded(&self) -> impl Future<Output = ()> {
        let mut messages = self.messages();

        async move {
            while let Some(message) = messages.next().await {
                if let Some(_) = message.loaded() {
                    return;
                }
            }

            unreachable!();
        }
    }
}



#[derive(Debug)]
pub struct Message {
    worker: DedicatedWorkerGlobalScope,
    js: JsMessage,
}

impl Message {
    pub fn name(&self) -> String {
        self.js.call()
    }

    pub fn with_args<A, R, F>(&self, f: F) -> R
        where A: Arguments,
              F: FnOnce(A) -> R {

        f(A::from_vec(self.js.args()))
    }

    pub fn send<A>(&self, value: A) where A: MessageSend {
        let id = self.js.id();

        let transfer = Array::new();

        let value = A::into_js(value, &transfer);

        let output: JsMessage = Object::new().unchecked_into();
        output.set_id(id);
        output.set_value(&value);

        self.worker.post_message_with_transfer(&output, &transfer).unwrap_throw();
    }
}


#[derive(Debug)]
pub struct SpawnedWorker {
    worker: DedicatedWorkerGlobalScope,
}

impl SpawnedWorker {
    pub fn new() -> Self {
        Self {
            worker: global().unchecked_into(),
        }
    }

    fn messages(&self) -> WorkerMessages {
        let (sender, receiver) = unbounded();

        WorkerMessages {
            _listener: EventListener::new(&self.worker, "message", move |e| {
                let e: &MessageEvent = e.unchecked_ref();
                sender.unbounded_send(e.data().unchecked_into()).unwrap_throw();
            }),
            receiver,
        }
    }

    pub fn listen<A, F>(&self, mut f: F) -> impl Future<Output = ()>
        where A: Future<Output = ()>,
              F: FnMut(Message) -> A {

        // This must be outside of the async block so that way it adds
        // the event listener immediately instead of on the next tick
        let messages = self.messages();

        let worker = self.worker.clone();

        async move {
            messages.for_each_concurrent(None, move |message| {
                f(Message {
                    worker: worker.clone(),
                    js: message,
                })
            }).await;
        }
    }

    pub fn set_loaded(&self) {
        let message: JsMessage = Object::new().unchecked_into();
        message.set_loaded(true);
        self.worker.post_message(&message).unwrap_throw();
    }
}
