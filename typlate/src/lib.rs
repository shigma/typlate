#![doc = include_str!("../README.md")]

use std::fmt::Display;
use std::str::FromStr;

mod string;

#[cfg(feature = "derive")]
pub use typlate_derive::TemplateParams;

pub use crate::string::{TemplateString, TemplateStringParams};

pub trait TemplateParams {
    type Template;

    fn format_template<'i>(&'i self, template: &'i Self::Template) -> impl Display;
}

pub struct Template<T: TemplateParams>(T::Template);

impl<T: TemplateParams> Template<T> {
    /// Format the template with the provided parameter values.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use typlate::{Template, TemplateParams};
    /// #[derive(TemplateParams)]
    /// struct Data {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// let template: Template<Data> = "Point: ({x}, {y})".parse().unwrap();
    /// let data = Data { x: 10, y: 20 };
    /// assert_eq!(template.format(&data), "Point: (10, 20)");
    /// ```
    pub fn format(&self, params: &T) -> String {
        params.format_template(&self.0).to_string()
    }
}

impl<T: TemplateParams> FromStr for Template<T>
where
    T::Template: FromStr,
{
    type Err = <T::Template as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Template(s.parse()?))
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    impl<T: TemplateParams> Serialize for Template<T>
    where
        T::Template: Serialize,
    {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.0.serialize(serializer)
        }
    }

    impl<'de, T: TemplateParams> Deserialize<'de> for Template<T>
    where
        T::Template: Deserialize<'de>,
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            Ok(Template(T::Template::deserialize(deserializer)?))
        }
    }
}
