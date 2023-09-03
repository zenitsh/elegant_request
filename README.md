# Elegant Request
## Features
Write your requests into a yaml file.

It's so easy!

```yaml
color: !Get
  url: http://127.0.0.1:3000/dog
  path:
    - !Const X
  params: {}
  value: color
person: !Get
  url: http://127.0.0.1:3000/dog
  path:
    - !Const X
  params: {}
  value: owner
home: !Get
  url: http://127.0.0.1:3000/home
  path: []
  params:
    person: !Ref person
  value: data.location
```
## Syntax
```
<resource_name>: [ !Get | !Post ]
  url: <url>
  args:
    <arg_name>: [ !Const <value> | !Ref <ref> ]
  value: [<name>.]*<name>
```
## Usage
```rust
//...
let mut r = ResponsePool::new(Request::new("/path/to/yaml"));
r.set_data_value("x", serde_json::Value::Number(serde_json::Number::from(123)));
let res = r.get("foo");
//...
```