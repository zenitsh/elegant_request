# Elegant Request
## Features
Write your requests into a yaml file.

It's so easy!

```yaml
foo: !Get
  url: <url>
  args:
    a: !Const 1
    b: !Const abc
  value: data.value
bar: !Get
  url: <url>
  args:
    t: !Ref foo
  value: !None
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