use serde::Deserialize;
use typlate::{TemplateParams, TemplateString};

#[derive(TemplateParams)]
struct Foo {
    bar: u32,
    qux: String,
}

#[derive(Deserialize)]
struct Messages {
    foo: TemplateString<Foo>,
}

#[test]
fn test_basic_formatting() {
    let template = TemplateString::<Foo>::parse("Hello {bar}, welcome {qux}!").unwrap();

    let params = Foo {
        bar: 42,
        qux: "world".to_string(),
    };

    assert_eq!(template.format(&params), "Hello 42, welcome world!");
}

#[test]
fn test_escaped_brackets() {
    let template = TemplateString::<Foo>::parse("{{bar}} is {bar}, {{{{qux}}}} is {{{qux}}}").unwrap();

    let params = Foo {
        bar: 42,
        qux: "test".to_string(),
    };

    assert_eq!(template.format(&params), "{bar} is 42, {{qux}} is {test}");
}

#[test]
fn test_deserialization() {
    let json = r#"{
        "foo": "Value is {bar}"
    }"#;

    let messages: Messages = serde_json::from_str(json).unwrap();

    let params = Foo {
        bar: 100,
        qux: "Alice".to_string(),
    };

    assert_eq!(messages.foo.format(&params), "Value is 100");
}

#[test]
fn test_invalid_field_error() {
    let json = r#"{
        "foo": "Value is {invalid_field}"
    }"#;

    let result: Result<Messages, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
