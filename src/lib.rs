pub mod request;

#[cfg(test)]
mod tests {

    use serde_json::Number;

    use crate::request::{Request, ResponsePool};

    #[test]
    fn it_works() {
        let request = Request::load_from_file("./res/example.yaml").unwrap();

        let mut response_pool = ResponsePool::new(request).unwrap();

        response_pool.set_data_value("input", serde_json::Value::Number(Number::from(2)));

        let c = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(response_pool.get("home"));
        println!("{:?}", c);
        let c = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(response_pool.get("color"));
        println!("{:?}", c);
        let c = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(response_pool.get("person"));
        println!("{:?}", c);
        println!("{:?}", response_pool);
    }
}
