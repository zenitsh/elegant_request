use std::{collections::HashMap, error::Error};

use reqwest::Method;
use serde_derive::{Deserialize, Serialize};

use async_recursion::async_recursion;

#[derive(Debug, Clone)]
pub enum Index {
    String(String),
    Integer(usize),
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
                    Err(_) => Index::String(s),
                }
            }));
            ValueName {
                names: Some(result),
            }
        }
    }
    pub fn parse<'a>(&'a self, s: &'a serde_json::Value) -> Result<&'a serde_json::Value, String> {
        let mut v = s;
        if let Some(v1) = self.names.clone() {
            for n in v1.iter() {
                v = match n {
                    Index::Integer(v2) => v.get(v2).ok_or(format!("No key: {}", v2))?,
                    Index::String(v2) => v.get(v2).ok_or(format!("No key: {}", v2))?,
                };
            }
        }
        Ok(v)
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
    path: Vec<RequestArgument>,
    params: HashMap<String, RequestArgument>,
    url: String,
    value_name: ValueName,
}

#[derive(Serialize, Deserialize)]
struct RequestYamlStruct {
    params: HashMap<String, RequestArgument>,
    path: Vec<RequestArgument>,
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
        path: Vec<RequestArgument>,
        params: HashMap<String, RequestArgument>,
        url: &str,
        value_name: ValueName,
    ) -> Self {
        Request {
            method,
            path,
            params,
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
                Request::new(
                    m,
                    r.path.clone(),
                    r.params.clone(),
                    &r.url,
                    ValueName::from_str(&r.value),
                ),
            )
        });
        Ok(HashMap::from_iter(map))
    }

    pub fn value_name(&self) -> &ValueName {
        &self.value_name
    }
}

#[derive(Debug)]
pub struct ResponsePool {
    data: HashMap<String, serde_json::Value>,
    cache: HashMap<String, serde_json::Value>,
    request: HashMap<String, Request>,
    client: reqwest::Client,
}

impl ResponsePool {
    pub fn new(request: HashMap<String, Request>) -> Result<Self, String> {
        Ok(ResponsePool {
            data: HashMap::new(),
            cache: HashMap::new(),
            request,
            client: reqwest::ClientBuilder::new()
                .cookie_store(true)
                .build().map_err(|err| { err.to_string() })?,
        })
    }
    pub fn clear_data(&mut self) {
        self.data.clear();
    }
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    pub fn set_data_value(&mut self, name: &str, value: serde_json::Value) {
        self.data.insert(String::from(name), value);
    }
    pub fn data_value(&self, name: &str) -> Option<serde_json::Value> {
        self.data.get(name).map(|val| { val.clone() })
    }
    #[async_recursion]
    pub async fn eval(&mut self, v: &RequestArgument) -> Result<String, Box<dyn Error>> {
        let res = match v {
            RequestArgument::Const(v) => v.clone(),
            RequestArgument::Ref(v) => self.get(&v).await?,
        };
        let res = match res {
            serde_json::Value::String(s) => s,
            s => s.to_string(),
        };
        Ok(res)
    }
    #[async_recursion]
    pub async fn get(&mut self, name: &str) -> Result<serde_json::Value, Box<dyn Error>> {
        if let Some(v) = self.data.get(name) {
            Ok(v.clone())
        } else {
            let r = self.request.get(name).ok_or("Request not exists.")?.clone();
            let mut params = HashMap::new();
            for (n, v) in r.params.iter() {
                params.insert(n.clone(), self.eval(v).await?);
            }
            let params = params;
            let mut path = Vec::new();
            for s in r.path.iter() {
                path.push(self.eval(s).await?);
            }
            let path = path.iter().fold(String::from(&r.url), |a, b| a + "/" + b);
            let url = if params.len() > 0 {
                reqwest::Url::parse_with_params(&path, &params)?
            } else {
                reqwest::Url::parse(&path)?
            };
            let key = url.to_string();
            let res = if let Some(res) = self.cache.get(&key) {
                res.clone()
            } else {
                println!("Request:{}", url);
                let req = self.client.request(r.method.clone(), url);
                let res: serde_json::Value = req.send().await?.json().await?;
                self.cache.insert(key, res.clone());
                res
            };
            let res = r.value_name().parse(&res)?;
            self.set_data_value(name, res.clone());
            Ok(res.clone())
        }
    }
    pub fn client<'a>(&'a self) -> &'a reqwest::Client {
        &self.client
    }
}
