use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de};
#[cfg(feature = "derive")]
pub use typlate_derive::TemplateParams;

pub trait TemplateParams {
    const FIELDS: &'static [&'static str];

    fn get_field(&self, index: usize) -> String;
}

#[derive(Debug, Clone, PartialEq)]
enum TemplateElement {
    Text(String),
    Var(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateString<T> {
    elements: Vec<TemplateElement>,
    _phantom: PhantomData<T>,
}

impl<T: TemplateParams> TemplateString<T> {
    pub fn format(&self, params: &T) -> String {
        let mut result = String::new();

        for element in &self.elements {
            match element {
                TemplateElement::Text(text) => result.push_str(text),
                TemplateElement::Var(index) => result.push_str(&params.get_field(*index)),
            }
        }

        result
    }
}

impl<T: TemplateParams> FromStr for TemplateString<T> {
    type Err = String;

    fn from_str(template: &str) -> Result<Self, Self::Err> {
        let mut elements = vec![];
        let mut chars = template.chars().peekable();
        let mut text = String::new();

        'outer: while let Some(char) = chars.next() {
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

                    let mut name = String::new();
                    for char in chars.by_ref() {
                        if char == '}' {
                            let index = T::FIELDS
                                .iter()
                                .position(|&f| f == name)
                                .ok_or_else(|| format!("Unknown field name: {}", name))?;
                            elements.push(TemplateElement::Var(index));
                            continue 'outer;
                        } else {
                            name.push(char);
                        }
                    }
                    return Err("Unclosed bracket in template".to_string());
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
}

impl<T: TemplateParams> fmt::Display for TemplateString<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for element in &self.elements {
            match element {
                TemplateElement::Text(text) => {
                    for char in text.chars() {
                        match char {
                            '{' => write!(f, "{{{{")?,
                            '}' => write!(f, "}}}}")?,
                            _ => write!(f, "{}", char)?,
                        }
                    }
                }
                TemplateElement::Var(index) => write!(f, "{{{}}}", T::FIELDS[*index])?,
            }
        }
        Ok(())
    }
}

impl<T: TemplateParams> Serialize for TemplateString<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
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

        impl<'de, T: TemplateParams> de::Visitor<'de> for TemplateStringVisitor<T> {
            type Value = TemplateString<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a template string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                TemplateString::from_str(value).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(TemplateStringVisitor { _phantom: PhantomData })
    }
}
