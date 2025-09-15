use std::fmt;
use std::marker::PhantomData;

use serde::de::{self, Deserialize, Deserializer, Visitor};
#[cfg(feature = "derive")]
pub use typlate_derive::TemplateParams;

pub trait TemplateParams {
    const FIELDS: &'static [&'static str];

    fn get_field(&self, index: usize) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq)]
enum TemplateElement {
    Text(String),
    Var(usize, &'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateString<T> {
    elements: Vec<TemplateElement>,
    _phantom: PhantomData<T>,
}

impl<T: TemplateParams> TemplateString<T> {
    pub fn parse(template: &str) -> Result<Self, String> {
        let mut elements = vec![];
        let mut chars = template.chars().peekable();
        let mut text = String::new();

        while let Some(char) = chars.next() {
            match char {
                '{' => {
                    if chars.peek() == Some(&'{') {
                        chars.next();
                        text.push('{');
                        continue;
                    }

                    if !text.is_empty() {
                        elements.push(TemplateElement::Text(text.clone()));
                        text.clear();
                    }

                    let mut field_name = String::new();
                    let mut is_closed = false;

                    while let Some(char) = chars.next() {
                        if char == '}' {
                            is_closed = true;
                            break;
                        } else {
                            field_name.push(char);
                        }
                    }
                    if !is_closed {
                        return Err("Unclosed bracket in template".to_string());
                    }

                    let index = T::FIELDS
                        .iter()
                        .position(|&f| f == field_name)
                        .ok_or_else(|| format!("Unknown field name: {}", field_name))?;
                    elements.push(TemplateElement::Var(index, T::FIELDS[index]));
                }
                '}' => {
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        text.push('}');
                    } else {
                        return Err("Unmatched closing bracket".to_string());
                    }
                }
                _ => text.push(char),
            }
        }

        if !text.is_empty() {
            elements.push(TemplateElement::Text(text));
        }
        Ok(Self {
            elements,
            _phantom: PhantomData,
        })
    }

    pub fn format(&self, params: &T) -> String {
        let mut result = String::new();

        for element in &self.elements {
            match element {
                TemplateElement::Text(text) => result.push_str(text),
                TemplateElement::Var(index, _) => match params.get_field(*index) {
                    Some(value) => result.push_str(&value),
                    None => result.push_str(format!("{{{}}}", T::FIELDS[*index]).as_str()),
                },
            }
        }

        result
    }
}

impl<'de, T: TemplateParams> Deserialize<'de> for TemplateString<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TemplateStringVisitor<T> {
            _phantom: PhantomData<T>,
        }

        impl<'de, T: TemplateParams> Visitor<'de> for TemplateStringVisitor<T> {
            type Value = TemplateString<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a template string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                TemplateString::parse(value).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(TemplateStringVisitor { _phantom: PhantomData })
    }
}
