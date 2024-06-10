use std::{borrow::Cow, collections::HashSet};

use proc_macro2::{Punct, Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Token,
    Fields, LitStr, Meta, PatPath, Path, PathSegment, Variant,
};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Ident, Token};
use tracing::{span::Id, Value};

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

struct AttrArgs {
    path: syn::Path,
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

        Ok(Self { path: level })
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

struct FieldAttr {
    args: Punctuated<FieldAttrArg, Token![,]>,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

#[derive(Hash, Debug)]
enum FieldAttrArg {
    DisplayMode(TracingDisplayMode),
    Rename(FieldAttrArgRename),
}

#[derive(Hash, Debug)]
struct FieldAttrArgRename {
    rename: Ident,
    eq_token: Token![=],
    expr: LitStr,
}

impl Parse for FieldAttrArgRename {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let rename: Ident = input.parse()?;

        assert_eq!(&rename.to_string(), "rename");

        let eq_token = input.parse()?;
        let expr = input.parse()?;

        Ok(Self {
            rename,
            eq_token,
            expr,
        })
    }
}

impl Parse for FieldAttrArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Do not do this.
        if input.fork().parse::<TracingDisplayMode>().is_ok() {
            return Ok(Self::DisplayMode(input.parse::<TracingDisplayMode>()?));
        }

        // Do not do this.
        if input.fork().parse::<FieldAttrArgRename>().is_ok() {
            return Ok(Self::Rename(input.parse::<FieldAttrArgRename>()?));
        }

        Err(input.error("unrecognized argument"))
    }
}

mod tracing_fields;

mod tracing_field;

#[derive(Hash, Debug, Copy, Clone, PartialEq, Eq)]
enum TracingDisplayMode {
    Value,
    Display,
    Debug,
    Skip,
}

impl Parse for TracingDisplayMode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let maybe_ident: Option<Ident> = input.parse()?;

        let name = maybe_ident.map(|ident| ident.to_string());

        let mode = match name.as_deref() {
            Some("display") => Ok(Self::Display),
            Some("debug") => Ok(Self::Debug),
            Some("value") | None => Ok(Self::Value),
            Some("skip") => Ok(Self::Skip),
            Some(_) => Err(input.error("invalid display arg")),
        };

        mode
    }
}

impl ToTokens for TracingDisplayMode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Value | Self::Skip => (),
            Self::Display => tokens.append(Punct::new('%', proc_macro2::Spacing::Alone)),
            Self::Debug => tokens.append(Punct::new('?', proc_macro2::Spacing::Alone)),
        }
    }
}

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

trait FindAttribute<'attr>: IntoIterator<Item = &'attr Attribute> + Sized {
    fn find_attribute(self, name: &str) -> Option<&'attr Attribute> {
        self.into_iter().find(|attr| {
            attr.path()
                .get_ident()
                .is_some_and(|ident| &ident.to_string() == name)
        })
    }

    fn contains_attribute(self, name: &str) -> bool {
        self.find_attribute(name).is_some()
    }
}

impl<'attr, It> FindAttribute<'attr> for It where It: IntoIterator<Item = &'attr Attribute> {}
