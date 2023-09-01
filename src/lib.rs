pub mod request;

#[cfg(test)]
mod tests {

    use serde_json::Number;

    use crate::request::{Request, ResponsePool};

    #[test]
    fn it_works() {
        let request =
            Request::load_from_file("./res/example.yaml").unwrap();

        let mut response_pool = ResponsePool::new(request);

        response_pool.set_data_value("input", serde_json::Value::Number(Number::from(2)));

        let c = response_pool.get("foo").unwrap();

        println!("{:?}", c);
    }
}
