worker::api! {
    pub async fn add(x: u32, y: u32) -> u32 {
        x + y
    }
}
