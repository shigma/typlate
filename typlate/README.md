# typlate
 
[![Crates.io](https://img.shields.io/crates/v/typlate.svg)](https://crates.io/crates/typlate)
[![Documentation](https://docs.rs/typlate/badge.svg)](https://docs.rs/typlate)
 
A Rust library for type-safe string templates with compile-time validation.

## Installation
 
Add to your `Cargo.toml`:
 
```toml
[dependencies]
typlate = { version = "0.1", features = ["derive"] }
```

## Quick Start

```rs
use typlate::{TemplateParams, TemplateString};

#[derive(TemplateParams)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    // Create a template string
    let template: TemplateString<User> = "Hello {name}, you are {age} years old!".parse().unwrap();

    // Create an instance with actual values
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    // Format the template with the values
    assert_eq!(template.format(&user), "Hello Alice, you are 30 years old!");
}
```

## Template Syntax

- Variables are enclosed in curly braces: `{variable_name}`
- To include literal braces, double them: `{{` for `{` and `}}` for `}`
- Variable names must match the field names of the target type

## Serde Support

Templates can be serialized and deserialized using serde:

```rs
use typlate::{TemplateParams, TemplateString};
use serde::{Serialize, Deserialize};

#[derive(TemplateParams)]
struct Data {
    value: u32,
}

#[derive(Serialize, Deserialize)]
struct Messages {
    foo: TemplateString<Data>,
}

let json = r#"{"foo": "Value is {value}"}"#;
let messages: Messages = serde_json::from_str(json).unwrap();
let data = Data { value: 42 };

assert_eq!(messages.foo.format(&data), "Value is 42");
```

## Error Handling

Template parsing will fail if:

- Variable names don't match any field in the target type
- Brackets are not properly matched
