use attribute_derive::FromAttr;
use proc_macro2::TokenStream;

use quote::{quote, ToTokens};

use crate::tracing_error;

use tracing_error::tracing::TracingPrintLevel;
use tracing_error::FieldAttribute;

use syn::{spanned::Spanned, Token};

use syn::Expr;

use syn::Ident;

use std::borrow::Cow;

#[derive(Debug, Clone)]
enum FieldName<'id> {
    Ident(Cow<'id, syn::Ident>),
    String(String),
}

impl std::fmt::Display for FieldName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => std::fmt::Display::fmt(ident, f),
            Self::String(string) => std::fmt::Display::fmt(string, f),
        }
    }
}

impl<'id> From<(&'id syn::Ident, Option<String>)> for FieldName<'id> {
    fn from(value: (&'id syn::Ident, Option<String>)) -> Self {
        if let Some(rename) = value.1 {
            rename.into()
        } else {
            value.0.into()
        }
    }
}

impl From<(syn::Ident, Option<String>)> for FieldName<'_> {
    fn from(value: (syn::Ident, Option<String>)) -> Self {
        if let Some(rename) = value.1 {
            rename.into()
        } else {
            value.0.into()
        }
    }
}

impl<'f> TryFrom<(&'f syn::Field, Option<String>)> for FieldName<'f> {
    type Error = syn::Error;

    fn try_from(value: (&'f syn::Field, Option<String>)) -> Result<Self, Self::Error> {
        let (field, rename) = value;

        if field.ident.is_none() && rename.is_none() {
            return Err(syn::Error::new(
                field.span(),
                "this field needs to have a name",
            ));
        }

        if let Some(rename) = rename {
            Ok(Self::from(rename))
        } else {
            Ok(Self::from(field.ident.as_ref().unwrap()))
        }
    }
}

impl TryFrom<(syn::Field, Option<String>)> for FieldName<'_> {
    type Error = syn::Error;

    fn try_from(value: (syn::Field, Option<String>)) -> Result<Self, Self::Error> {
        let (field, rename) = value;

        if field.ident.is_none() && rename.is_none() {
            return Err(syn::Error::new(
                field.span(),
                "this field needs to have a name",
            ));
        }

        if let Some(rename) = rename {
            Ok(Self::from(rename))
        } else {
            Ok(Self::from(field.ident.unwrap()))
        }
    }
}

impl<'f> TryFrom<&'f syn::Field> for FieldName<'f> {
    type Error = syn::Error;

    fn try_from(field: &'f syn::Field) -> Result<Self, Self::Error> {
        field.ident.as_ref().map_or(
            Err(syn::Error::new(
                field.span(),
                "this field needs an identifier!",
            )),
            |ident| Ok(Self::from(ident)),
        )
    }
}

impl<'id> From<&'id syn::Ident> for FieldName<'id> {
    fn from(ident: &'id syn::Ident) -> Self {
        Self::Ident(Cow::Borrowed(ident))
    }
}

impl From<syn::Ident> for FieldName<'_> {
    fn from(ident: syn::Ident) -> Self {
        Self::Ident(Cow::Owned(ident))
    }
}

impl From<String> for FieldName<'_> {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<usize> for FieldName<'_> {
    fn from(value: usize) -> Self {
        value.to_string().into()
    }
}

impl quote::ToTokens for FieldName<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::String(string) => string.to_tokens(tokens),
        }
    }
}

/* pub(crate) struct Field<'id> {
    pub(crate) name: FieldName<'id>,
    rename: Option<LitStr>,
    // =
    pub(crate) value: TracingValue,

    pub(crate) span: Span,
}

impl<'i> Field<'i> {
    pub(crate) fn new(
        display_mode: TracingPrintLevel,
        name: FieldName<'i>,
        rename: Option<LitStr>,
        span: Span,
    ) -> Self {
        todo!();

        //Self {
        //    name,
        //    rename,
        //    value: TracingValue::new_self_field(display_mode, field_name),
        //}
    }

    pub(crate) fn new_cow(
        display_mode: TracingPrintLevel,
        name: Cow<'i, Ident>,
        rename: Option<LitStr>,
        span: Span,
    ) -> Self {
        Self {
            name: name.clone(),
            rename,
            value: TracingValue::new_self_field(display_mode, name.to_owned().into_owned()),
            span,
        }
    }

    pub(crate) fn new_numbered(display_mode: TracingPrintLevel, index: usize, span: Span) -> Self {
        Self::new_cow(
            display_mode,
            Cow::Owned(Ident::new(&index.to_string(), Span::call_site())),
            None,
            span,
        )
    }

    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    pub(crate) fn skip_displaying(&self) -> bool {
        self.value.print == TracingPrintLevel::Skip
    }
}

impl<'f: 'i, 'i> TryFrom<&'f syn::Field> for Field<'i> {
    type Error = syn::Error;

    fn try_from(value: &'f syn::Field) -> Result<Self, Self::Error> {
        let span = value.span();

        let ident = value
            .ident
            .clone()
            .ok_or(syn::Error::new(span, "field has no name"))?;

        let attr: crate::attributes::Field = value.attrs.as_slice().try_into().unwrap_or_default();
        let rename = attr.rename().map(syn::LitStr::from);
        let display_mode = attr
            .print_level()
            .map(TracingPrintLevel::from)
            .unwrap_or_default();

        Ok(Self::new_cow(display_mode, Cow::Owned(ident), rename, span))
    }
}

impl TryFrom<syn::Field> for Field<'_> {
    type Error = syn::Error;

    fn try_from(value: syn::Field) -> Result<Self, Self::Error> {
        let span = value.span();

        let ident = value
            .ident
            .clone()
            .ok_or(syn::Error::new(span, "field has no name"))?;

        let attr: crate::attributes::Field = value.attrs.try_into().unwrap_or_default();
        let rename = attr.rename().map(syn::LitStr::from);
        let display_mode = attr
            .print_level()
            .map(TracingPrintLevel::from)
            .unwrap_or_default();

        Ok(Self::new_cow(display_mode, Cow::Owned(ident), rename, span))
    }
}

impl ToTokens for Field<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.skip_displaying() {
            let name = self
                .rename
                .as_ref()
                .map_or_else(|| self.name().to_string(), |lit| lit.value());
            let value = &self.value;

            let expanded = quote_spanned! { self.span=> #name = #value };
            tokens.append_all(expanded);
        }
    }
}
 */
pub struct TracingValue {
    print: TracingPrintLevel,
    expr: Expr,
}

impl TracingValue {
    pub fn new(print: TracingPrintLevel, expr: Expr) -> Self {
        Self { print, expr }
    }

    pub fn new_self_field(print: TracingPrintLevel, field_name: Ident) -> Self {
        let expr = Expr::Field(syn::ExprField {
            attrs: Vec::new(),
            base: Box::new(Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: syn::parse_str("self").unwrap(),
            })),
            dot_token: <Token![.]>::default(),
            member: syn::Member::Named(field_name),
        });

        Self::new(print, expr)
    }
}

impl ToTokens for TracingValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expr = &self.expr;
        use TracingPrintLevel as P;
        match &self.print {
            P::Display => quote! { %#expr }.to_tokens(tokens),
            P::Debug => quote! { ?#expr }.to_tokens(tokens),
            P::CustomDebug(lit) => syn::Macro {
                path: syn::parse_str("format").unwrap(),
                bang_token: <Token![!]>::default(),
                delimiter: syn::MacroDelimiter::Paren(syn::token::Paren::default()),
                tokens: {
                    let format = format!("{{{}}}", lit.value());
                    quote! { #format, #expr }
                },
            }
            .to_tokens(tokens),
            P::Value => expr.to_tokens(tokens),
            _ => (),
        }
    }
}

fn new_self_field(member: syn::Member) -> syn::ExprField {
    syn::ExprField {
        attrs: Vec::new(),
        base: Box::new(syn::parse_str("self").unwrap()),
        dot_token: <syn::Token![.]>::default(),
        member,
    }
}

pub struct Full<'f> {
    name: FieldName<'f>,
    // =
    value: TracingValue,
}

impl TryFrom<syn::Field> for Full<'_> {
    type Error = syn::Error;

    fn try_from(field: syn::Field) -> Result<Self, Self::Error> {
        let attr = FieldAttribute::from_attributes(&field.attrs)?;

        let field_name = FieldName::try_from((field.clone(), attr.rename.as_ref().cloned()))?;

        if let Some(ident) = &field.ident {
            let value = TracingValue::new_self_field(attr.print, ident.clone());
            Ok(Self::new(field_name, value))
        } else {
            Err(Self::Error::new(
                field.span(),
                "this field needs an identifier",
            ))
        }
    }
}

impl<'f> Full<'f> {
    fn new(name: FieldName<'f>, value: TracingValue) -> Self {
        Self { name, value }
    }

    pub fn new_numbered(n: usize, print_level: TracingPrintLevel) -> Self {
        Self::new(
            n.into(),
            TracingValue::new(
                print_level,
                syn::Expr::Field(new_self_field(syn::Member::Unnamed(n.into()))),
            ),
        )
    }

    fn new_self_field(ident: &'f syn::Ident, print_level: TracingPrintLevel) -> Self {
        Self::new(
            ident.into(),
            TracingValue::new_self_field(print_level, ident.clone()),
        )
    }
}

impl quote::ToTokens for Full<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let value = &self.value;
        quote! { #name = #value }.to_tokens(tokens)
    }
}

impl<'a> TryFrom<&'a syn::Field> for Full<'a> {
    type Error = syn::Error;

    fn try_from(value: &'a syn::Field) -> Result<Self, Self::Error> {
        Ok(Self::new_self_field(
            value.ident.as_ref().unwrap(),
            FieldAttribute::from_attributes(&value.attrs)?.print,
        ))
    }
}
