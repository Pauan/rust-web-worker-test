worker::api! {
    pub async fn compute(input: Vec<u8>) -> Vec<u8> {
        let mut input = input;
        input[0] = 5;
        input
    }
}
