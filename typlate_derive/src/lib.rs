use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, LitStr, parse_macro_input};

#[proc_macro_derive(TemplateParams)]
pub fn derive_template_params(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| f.ident.clone().unwrap())
                .collect::<Vec<_>>(),
            Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("{}", i))
                .collect::<Vec<_>>(),
            Fields::Unit => vec![],
        },
        _ => panic!("TemplateParams can only be derived for structs"),
    };

    let field_names = fields
        .iter()
        .map(|field| LitStr::new(&field.to_string(), Span::call_site().into()));

    let field_matches = fields.iter().enumerate().map(|(index, field)| {
        quote! { #index => Some(self.#field.to_string()), }
    });

    quote! {
        impl TemplateParams for #ident {
            const FIELDS: &'static [&'static str] = &[#(#field_names),*];

            fn get_field(&self, index: usize) -> Option<String> {
                match index {
                    #(#field_matches)*
                    _ => None,
                }
            }
        }
    }
    .into()
}
