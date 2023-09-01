use std::{collections::HashMap, error::Error};

use reqwest::{Method, Response};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Index {
    String(String),
    Integer(usize)
}

#[derive(Debug, Clone)]
pub struct ValueName {
    names: Option<Vec<Index>>,
}

impl ValueName {
    pub fn from_str(s: &str) -> ValueName {
        if s == "" {
            ValueName { names: None }
        } else {
            let result = s.split(".").map(|n| String::from(n));
            let result = Vec::from_iter(result.map(|s| {
                let t = usize::from_str_radix(&s, 10);
                match t {
                    Ok(v) => Index::Integer(v),
                    Err(_) => Index::String(s)
                }
            }));
            ValueName {
                names: Some(result),
            }
        }
    }
    pub fn parse<'a>(&'a self, s: &'a serde_json::Value) -> &'a serde_json::Value {
        let mut v = s;
        if let Some(v1) = self.names.clone() {
            for n in v1.iter() {
                v = match n {
                    Index::Integer(v2) => v.get(v2),
                    Index::String(v2) => v.get(v2)
                }.unwrap();
            }
        }
        v
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RequestArgument {
    Ref(String),
    Const(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    args: HashMap<String, RequestArgument>,
    url: String,
    value_name: ValueName,
}

#[derive(Serialize, Deserialize)]
struct RequestYamlStruct {
    args: HashMap<String, RequestArgument>,
    url: String,
    value: String,
}

#[derive(Serialize, Deserialize)]
enum RequestYamlEnum {
    //Put(RequestYamlStruct),
    //Delete(RequestYamlStruct),
    Get(RequestYamlStruct),
    Post(RequestYamlStruct),
}

impl Request {
    pub fn new(
        method: Method,
        args: HashMap<String, RequestArgument>,
        url: &str,
        value_name: ValueName,
    ) -> Self {
        Request {
            method,
            args,
            url: String::from(url),
            value_name,
        }
    }
    pub fn load_from_file(
        path: &str,
    ) -> Result<HashMap<String, Request>, Box<dyn std::error::Error>> {
        let map: HashMap<String, RequestYamlEnum> =
            serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
        let map = map.iter().map(|(s, r)| {
            let (m, r) = match r {
                RequestYamlEnum::Get(r) => (Method::GET, r),
                RequestYamlEnum::Post(r) => (Method::POST, r),
            };
            (
                s.clone(),
                Request::new(m, r.args.clone(), &r.url, ValueName::from_str(&r.value)),
            )
        });
        Ok(HashMap::from_iter(map))
    }
    pub async fn send(&self, args: &HashMap<String, String>) -> Response {
        let client = reqwest::Client::new();
        let s = client.request(
            self.method.clone(),
            reqwest::Url::parse_with_params(&self.url, args).unwrap(),
        );
        s.send().await.unwrap()
    }
    pub fn value_name(&self) -> &ValueName {
        &self.value_name
    }
}

#[derive(Debug)]
pub struct ResponsePool {
    data: HashMap<String, serde_json::Value>,
    request: HashMap<String, Request>,
}

impl ResponsePool {
    pub fn new(request: HashMap<String, Request>) -> Self {
        ResponsePool {
            data: HashMap::new(),
            request,
        }
    }
    pub fn set_data_value(&mut self, name: &str, value: serde_json::Value) {
        self.data.insert(String::from(name), value);
    }
    pub fn data_value(&self, name: &str) -> serde_json::Value {
        self.data.get(name).unwrap().clone()
    }
    pub fn get(&mut self, name: &str) -> Result<serde_json::Value, Box<dyn Error>> {
        if let Some(v) = self.data.get(name) {
            Ok(v.clone())
        } else {
            let r = self.request.get(name).unwrap().clone();
            let mut args = HashMap::new();
            for (n, v) in r.args.iter() {
                match v {
                    RequestArgument::Const(v) => {
                        args.insert(n.clone(), v.to_string());
                        ()
                    }
                    RequestArgument::Ref(v) => {
                        args.insert(n.clone(), self.get(&v).unwrap().to_string());
                    }
                };
            }
            let req = r.send(&args);
            let res = tokio::runtime::Runtime::new().unwrap().block_on(req);
            let res: serde_json::Value = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(res.json())
                .unwrap();
            let res = r.value_name().parse(&res);
            self.set_data_value(name, res.clone());
            Ok(res.clone())
        }
    }
}
