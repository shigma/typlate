#![doc = include_str!("../README.md")]

use std::fmt::{self, Display, Write};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::str::FromStr;

use crate::TemplateParams;

/// A trait for types that can provide template parameters.
///
/// This trait is typically implemented using the `#[derive(TemplateParams)]` macro.
/// It provides the field names and values that can be used in templates.
pub trait TemplateStringParams {
    /// Array of field names available for use in templates.
    const FIELDS: &'static [&'static str];

    /// Format the field at the given index into the provided formatter.
    fn fmt_field(&self, f: &mut fmt::Formatter, index: usize) -> fmt::Result;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TemplateElement {
    Text(String),
    Var(usize),
}

/// A type-safe template string that can be formatted with values of type `T`.
///
/// Template strings contain placeholders in the form `{field_name}` that correspond
/// to fields in type `T`. The template is validated at parse time to ensure all
/// placeholders are valid.
///
/// ## Examples
///
/// ```
/// # use typlate::{Template, TemplateParams};
/// #[derive(TemplateParams)]
/// struct Person {
///     name: String,
///     title: String,
/// }
///
/// let template: Template<Person> = "Dear {title} {name}".parse().unwrap();
/// let person = Person {
///     name: "Smith".to_string(),
///     title: "Dr.".to_string(),
/// };
/// assert_eq!(template.format(&person), "Dear Dr. Smith");
/// ```
pub struct TemplateString<T> {
    elements: Vec<TemplateElement>,
    phantom: PhantomData<T>,
}

impl<T: TemplateStringParams> TemplateParams for T {
    type Template = TemplateString<Self>;

    fn format_template<'i>(&'i self, template: &'i Self::Template) -> impl Display {
        Parameterized(self, template)
    }
}

pub struct Parameterized<'i, T>(&'i T, &'i TemplateString<T>);

impl<'i, T: TemplateStringParams> fmt::Display for Parameterized<'i, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for element in &self.1.elements {
            match element {
                TemplateElement::Text(text) => f.write_str(text)?,
                TemplateElement::Var(index) => self.0.fmt_field(f, *index)?,
            }
        }
        Ok(())
    }
}

impl<T: TemplateStringParams> FromStr for TemplateString<T> {
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
                                .ok_or_else(|| format!("Unknown field name: {name}"))?;
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
            phantom: PhantomData,
        })
    }
}

impl<T> Clone for TemplateString<T> {
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for TemplateString<T> {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

impl<T> Eq for TemplateString<T> {}

impl<T> PartialOrd for TemplateString<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for TemplateString<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.elements.cmp(&other.elements)
    }
}

impl<T> Hash for TemplateString<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.elements.hash(state);
    }
}

impl<T: TemplateStringParams> fmt::Debug for TemplateString<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("TemplateString").field(&format!("{self}")).finish()
    }
}

impl<T: TemplateStringParams> fmt::Display for TemplateString<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for element in &self.elements {
            match element {
                TemplateElement::Text(text) => {
                    for char in text.chars() {
                        match char {
                            '{' => f.write_str("{{")?,
                            '}' => f.write_str("}}")?,
                            _ => f.write_char(char)?,
                        }
                    }
                }
                TemplateElement::Var(index) => {
                    f.write_char('{')?;
                    f.write_str(T::FIELDS[*index])?;
                    f.write_char('}')?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

    use super::*;

    impl<T: TemplateStringParams> Serialize for TemplateString<T> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(&self.to_string())
        }
    }

    impl<'de, T: TemplateStringParams> Deserialize<'de> for TemplateString<T> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_str(TemplateStringVisitor(PhantomData))
        }
    }

    struct TemplateStringVisitor<T>(PhantomData<T>);

    impl<'de, T: TemplateStringParams> de::Visitor<'de> for TemplateStringVisitor<T> {
        type Value = TemplateString<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a template string")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            value.parse().map_err(de::Error::custom)
        }
    }
}
