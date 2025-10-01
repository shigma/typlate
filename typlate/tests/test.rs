use serde::{Deserialize, Serialize};
use typlate::{Template, TemplateParams};

#[derive(TemplateParams)]
struct Foo<'i> {
    bar: u32,
    qux: &'i str,
}

#[derive(Deserialize, Serialize)]
struct Messages {
    foo: Template<Foo<'static>>,
}

#[test]
fn test_basic_formatting() {
    let template: Template<Foo> = "Hello {bar}, welcome {qux}!".parse().unwrap();
    let params = Foo { bar: 42, qux: "world" };

    assert_eq!(template.format(&params), "Hello 42, welcome world!");
}

#[test]
fn test_escaped_brackets() {
    let template: Template<Foo> = "{{bar}} is {bar}, {{{{qux}}}} is {{{qux}}}".parse().unwrap();
    let params = Foo { bar: 42, qux: "test" };

    assert_eq!(template.format(&params), "{bar} is 42, {{qux}} is {test}");
}

#[test]
fn test_serde() {
    let json = r#"{
        "foo": "Value is {bar}"
    }"#;

    let messages: Messages = serde_json::from_str(json).unwrap();
    let params = Foo { bar: 100, qux: "Alice" };

    assert_eq!(messages.foo.format(&params), "Value is 100");
    assert_eq!(serde_json::to_string(&messages).unwrap(), r#"{"foo":"Value is {bar}"}"#);
}

#[test]
fn test_invalid_field_error() {
    let json = r#"{
        "foo": "Value is {invalid_field}"
    }"#;

    let result: Result<Messages, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
