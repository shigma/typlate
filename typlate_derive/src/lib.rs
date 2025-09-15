use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, LitStr, Member, parse_macro_input};

#[proc_macro_derive(TemplateParams)]
pub fn derive_template_params(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let ident = &input.ident;

    let mut ident_names = vec![];
    let mut match_arms = vec![];

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                for (index, field) in fields.named.iter().enumerate() {
                    let ident = field.ident.as_ref().unwrap();
                    ident_names.push(LitStr::new(&ident.to_string(), field.span()));
                    match_arms.push(quote! { #index => self.#ident.to_string(), });
                }
            }
            Fields::Unnamed(fields) => {
                for (index, field) in fields.unnamed.iter().enumerate() {
                    let member = Member::Unnamed(index.into());
                    ident_names.push(LitStr::new(&index.to_string(), field.span()));
                    match_arms.push(quote! { #index => self.#member.to_string(), });
                }
            }
            Fields::Unit => {}
        },
        _ => panic!("TemplateParams can only be derived for structs"),
    }

    quote! {
        impl TemplateParams for #ident {
            const FIELDS: &'static [&'static str] = &[#(#ident_names),*];

            fn get_field(&self, index: usize) -> String {
                match index {
                    #(#match_arms)*
                    _ => panic!("Index out of bounds"),
                }
            }
        }
    }
    .into()
}
