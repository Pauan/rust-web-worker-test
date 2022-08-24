#[macro_export]
macro_rules! api {
    ($(pub async fn $name:ident($($arg:ident: $t:ty),*) -> $ret:ty $body:block)*) => {
        #[cfg(feature = "api")]
        pub mod api {
            use std::future::Future;

            #[derive(Debug)]
            pub struct Thread {
                worker: $crate::Worker,
            }

            impl Thread {
                pub fn new(url: &str) -> Self {
                    Self {
                        worker: $crate::Worker::new(url),
                    }
                }

                $(pub fn $name(&self, $($arg: $t),*) -> impl Future<Output = $ret> {
                    self.worker.call(stringify!($name))
                        $(.arg($arg))*
                        .send()
                })*
            }

            pub fn spawn(url: &str) -> impl Future<Output = Thread> {
                let thread = Thread::new(url);

                async move {
                    thread.worker.wait_loaded().await;
                    thread
                }
            }
        }

        #[cfg(feature = "api")]
        pub use api::spawn;


        #[cfg(not(feature = "api"))]
        use wasm_bindgen::prelude::*;

        #[cfg(not(feature = "api"))]
        #[wasm_bindgen(start)]
        pub async fn main_js() {
            #[cfg(debug_assertions)]
            console_error_panic_hook::set_once();

            let worker = $crate::SpawnedWorker::new();

            // This must be called before set_loaded, so that way we
            // guarantee that we don't miss any messages
            let listener = worker.listen(|message| {
                async move {
                    match message.name().as_str() {
                        $(stringify!($name) => {
                            let value = message.with_args(|($($arg,)*): ($($t,)*)| {
                                async move { $body }
                            }).await;

                            message.send::<$ret>(value)
                        },)*
                        _ => {
                            unreachable!();
                        },
                    }
                }
            });

            worker.set_loaded();

            listener.await;
        }
    };
}
