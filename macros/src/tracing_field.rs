use proc_macro2::TokenStream;

use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};

use crate::attributes::field::TracingPrintLevel;

use syn::{spanned::Spanned, LitStr, Token};

use proc_macro2::Span;

use syn::Expr;

use syn::Ident;

use std::borrow::Cow;

pub(crate) struct TracingField<'i> {
    pub(crate) name: Cow<'i, Ident>,
    rename: Option<LitStr>,
    pub(crate) value: TracingValue,
    pub(crate) span: Span,
}

impl<'i> TracingField<'i> {
    pub(crate) fn new(
        display_mode: TracingPrintLevel,
        name: &'i Ident,
        rename: Option<LitStr>,
        span: Span,
    ) -> Self {
        Self::new_cow(display_mode, Cow::Borrowed(name), rename, span)
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

impl<'f: 'i, 'i> TryFrom<&'f syn::Field> for TracingField<'i> {
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

impl TryFrom<syn::Field> for TracingField<'_> {
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

impl ToTokens for TracingField<'_> {
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

pub struct TracingValue {
    print: crate::attributes::field::TracingPrintLevel,
    expr: Expr,
}

impl TracingValue {
    pub fn new(print: crate::attributes::field::TracingPrintLevel, expr: Expr) -> Self {
        Self { print, expr }
    }

    pub fn new_self_field(
        print: crate::attributes::field::TracingPrintLevel,
        field_name: Ident,
    ) -> Self {
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
        use crate::attributes::field::TracingPrintLevel as P;
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
