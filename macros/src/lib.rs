use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Token};
use syn::{Fields, Path};

mod attributes;

#[proc_macro_derive(TracingError, attributes(event, field))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let event = &input
        .attrs
        .iter()
        .find(|attr| {
            attr.path()
                .get_ident()
                .is_some_and(|ident| &ident.to_string() == "event")
        })
        .expect("no event attr");

    let meta = event
        .parse_args::<attributes::event::MetaList>()
        .expect("attr syntax invalid");
    let level = &meta.level().unwrap().level;

    let function_body = match input.data {
        Data::Struct(data) => {
            let tracing_fields = tracing_fields::tracing_fields(&data.fields);
            let tracing_body = TracingEventExpr::new(level.clone(), Some(tracing_fields), true);

            quote! {
                tracing::event!(
                    #tracing_body
                )
            }
        }
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|variant| {
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        assert!(fields.unnamed.len() == 1);
                    }
                    _ => unimplemented!(),
                }

                let ident = &variant.ident;

                quote! { #ident(err) => TracingError::event(err) }
            });

            quote! {
                match self {
                    #(
                        Self::#variants
                    ),*
                }
            }
        }
        _ => unimplemented!(),
    };

    let name = input.ident;

    let expanded = quote! {
        impl TracingError for #name {
            fn event(&self) {
                #function_body
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

mod tracing_fields;

mod tracing_field;

struct TracingEventExpr<'a> {
    level: syn::ExprPath,
    fields: Option<tracing_fields::TracingFields<'a>>,
    display: bool,
}

impl<'a> TracingEventExpr<'a> {
    fn new(level: Path, fields: Option<tracing_fields::TracingFields<'a>>, display: bool) -> Self {
        Self {
            level: syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: level,
            },
            fields,
            display,
        }
    }
}

impl ToTokens for TracingEventExpr<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.level.to_tokens(tokens);

        if self.fields.is_some() || self.display {
            <Token![,]>::default().to_tokens(tokens);
        }

        if let Some(fields) = &self.fields {
            fields.to_tokens(tokens);

            if self.display {
                <Token![,]>::default().to_tokens(tokens);
            }
        }

        if self.display {
            quote! { "{}", self }.to_tokens(tokens);
        }
    }
}
