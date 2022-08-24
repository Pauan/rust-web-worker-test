use wasm_bindgen::prelude::*;


#[wasm_bindgen(start)]
pub async fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    console_log::init_with_level(log::Level::Debug).unwrap();


    {
        let thread1 = thread1::spawn("/js/thread1.js").await;

        let x = thread1.compute(vec![1, 2, 3, 4]).await;

        log::info!("{:#?}", x);
    }


    {
        let thread2 = thread2::spawn("/js/thread2.js").await;

        let x = thread2.add(4, 11).await;

        log::info!("{:#?}", x);
    }


    Ok(())
}
