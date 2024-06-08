use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Fields, Meta, Path, PathSegment, Variant,
};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Ident, Token};
use tracing::Value;

#[proc_macro_derive(TracingError, attributes(level, display, default))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let attr = &input
        .attrs
        .iter()
        .find(|attr| {
            attr.path()
                .get_ident()
                .is_some_and(|ident| &ident.to_string() == "level")
        })
        .expect("no level attr");

    let level = attr
        .parse_args::<AttrArgs>()
        .expect("attr syntax invalid")
        .level;

    let expanded = match input.data {
        Data::Struct(data) => {
            let fields = data.fields;
            match fields {
                Fields::Named(fields) => {
                    let fields = fields.named;
                    let field_values = fields.iter().map(|field| {
                        let name = field.ident.as_ref().unwrap();

                        if field.attrs.iter().any(|attr| {
                            attr.path()
                                .get_ident()
                                .is_some_and(|ident| &ident.to_string() == "display")
                        }) {
                            quote! { #name = tracing::field::display(self.#name) }
                        } else if field.attrs.iter().any(|attr| {
                            attr.path()
                                .get_ident()
                                .is_some_and(|ident| &ident.to_string() == "debug")
                        }) {
                            quote! { #name = tracing::field::debug(self.#name) }
                        } else {
                            quote! { #name = self.#name }
                        }
                    });
                    let name = input.ident;

                    quote! {
                        impl TracingError for #name {
                            fn event(&self) {
                                tracing::event!(
                                    #level,
                                    #(#field_values,)*
                                    "{}", self.to_string()
                                )
                            }
                        }
                    }
                }
                _ => todo!(),
            }
        }
        Data::Enum(data) => {
            let name = input.ident;

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

            //panic!(
            //   "{:?}",
            //    variants.map(|ts| ts.to_string()).collect::<Vec<_>>()
            //);

            quote! {
                impl TracingError for #name {
                    fn event(&self) {
                        match self {
                            #(
                                Self::#variants
                            ),*
                        }
                    }
                }
            }
        }
        _ => unimplemented!(),
    };

    proc_macro::TokenStream::from(expanded)
}

struct AttrArgs {
    level: syn::Path,
}

impl Parse for AttrArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: Path = input.parse()?;

        let level = if let Some(ident) = path.get_ident() {
            let mut path: Path = syn::parse_str("tracing::Level")?;
            path.segments.push(PathSegment {
                ident: ident.clone(),
                arguments: syn::PathArguments::None,
            });
            path
        } else {
            path
        };

        Ok(Self { level })
    }
}

#[derive(Default)]
enum TracingSymbol {
    #[default]
    Debug,
    Display,
    Value,
}

impl From<TracingSymbol> for Option<char> {
    fn from(value: TracingSymbol) -> Self {
        match value {
            TracingSymbol::Debug => Some('?'),
            TracingSymbol::Display => Some('%'),
            TracingSymbol::Value => None,
        }
    }
}

trait AsTracingSymbol: std::fmt::Debug {
    fn as_display(&self) -> Option<TracingSymbol> {
        None
    }

    fn as_value(&self) -> Option<TracingSymbol> {
        None
    }

    fn as_tracing_symbol(&self) -> TracingSymbol {
        self.as_value()
            .unwrap_or_else(|| self.as_display().unwrap_or_default())
    }
}

impl<T: tracing::Value + std::fmt::Debug> AsTracingSymbol for T {
    fn as_value(&self) -> Option<TracingSymbol> {
        Some(TracingSymbol::Value)
    }
}

/*impl<T: std::fmt::Display + std::fmt::Debug> AsTracingSymbol for T {
    fn as_display(&self) -> Option<TracingSymbol> {
        Some(TracingSymbol::Display)
    }
}*/
