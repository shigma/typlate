#![doc = include_str!("../README.md")]

use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, de};
#[cfg(feature = "derive")]
pub use typlate_derive::TemplateParams;

/// A trait for types that can provide template parameters.
///
/// This trait is typically implemented using the `#[derive(TemplateParams)]` macro.
/// It provides the field names and values that can be used in templates.
pub trait TemplateParams {
    /// Array of field names available for use in templates.
    const FIELDS: &'static [&'static str];

    /// Get the string representation of a field by its index.
    fn get_field(&self, index: usize) -> String;
}

#[derive(Debug, Clone, PartialEq)]
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
/// # use typlate::{TemplateParams, TemplateString};
/// #[derive(TemplateParams)]
/// struct Person {
///     name: String,
///     title: String,
/// }
///
/// let template: TemplateString<Person> = "Dear {title} {name}".parse().unwrap();
/// let person = Person {
///     name: "Smith".to_string(),
///     title: "Dr.".to_string(),
/// };
/// assert_eq!(template.format(&person), "Dear Dr. Smith");
/// ```
pub struct TemplateString<T> {
    elements: Vec<TemplateElement>,
    _phantom: PhantomData<T>,
}

impl<T: TemplateParams> TemplateString<T> {
    /// Format the template with the provided parameter values.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use typlate::{TemplateParams, TemplateString};
    /// #[derive(TemplateParams)]
    /// struct Data {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let template: TemplateString<Data> = "Point: ({x}, {y})".parse().unwrap();
    /// let data = Data { x: 10, y: 20 };
    /// assert_eq!(template.format(&data), "Point: (10, 20)");
    /// ```
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

// do not require T: Clone for Clone
impl<T> Clone for TemplateString<T> {
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
            _phantom: PhantomData,
        }
    }
}

// do not require T: Debug for Debug
impl<T> fmt::Debug for TemplateString<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TemplateString")
            .field("elements", &self.elements)
            .finish()
    }
}

// do not require T: PartialEq for PartialEq
impl<T> PartialEq for TemplateString<T> {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
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

#[cfg(feature = "serde")]
impl<T: TemplateParams> Serialize for TemplateString<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
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
