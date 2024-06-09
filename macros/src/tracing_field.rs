use proc_macro2::{Punct, TokenStream};

use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};

use crate::{attributes::field::ArgPrintLevel, rename_from_attrs};

use super::display_mode_from_attrs;

use syn::{spanned::Spanned, LitStr, Token};

use proc_macro2::Span;

use syn::Expr;

use syn::Ident;

use std::borrow::Cow;

use super::TracingDisplayMode;

pub(crate) struct TracingField<'i> {
    pub(crate) name: Cow<'i, Ident>,
    rename: Option<LitStr>,
    pub(crate) value: TracingValue,
    pub(crate) span: Span,
}

impl<'i> TracingField<'i> {
    pub(crate) fn new(
        display_mode: ArgPrintLevel,
        name: &'i Ident,
        rename: Option<LitStr>,
        span: Span,
    ) -> Self {
        Self::new_cow(display_mode, Cow::Borrowed(name), rename, span)
    }

    pub(crate) fn new_cow(
        display_mode: ArgPrintLevel,
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

    pub(crate) fn new_numbered(display_mode: ArgPrintLevel, index: usize, span: Span) -> Self {
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
        self.value.print == ArgPrintLevel::Skip
    }
}

impl<'f: 'i, 'i> TryFrom<&'f syn::Field> for TracingField<'i> {
    type Error = ();

    fn try_from(value: &'f syn::Field) -> Result<Self, Self::Error> {
        let ident = value.ident.as_ref().ok_or(())?;
        let rename = rename_from_attrs(&value.attrs);

        Ok(Self::new_cow(
            display_mode_from_attrs(&value.attrs),
            Cow::Borrowed(ident),
            rename,
            value.span(),
        ))
    }
}

impl TryFrom<syn::Field> for TracingField<'_> {
    type Error = ();

    fn try_from(value: syn::Field) -> Result<Self, Self::Error> {
        let ident = value.ident.clone().ok_or(())?;
        let rename = rename_from_attrs(&value.attrs);

        Ok(Self::new_cow(
            display_mode_from_attrs(&value.attrs),
            Cow::Owned(ident),
            rename,
            value.span(),
        ))
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
    print: crate::attributes::field::ArgPrintLevel,
    expr: Expr,
}

impl TracingValue {
    pub fn new(print: crate::attributes::field::ArgPrintLevel, expr: Expr) -> Self {
        Self { print, expr }
    }

    pub fn new_self_field(
        print: crate::attributes::field::ArgPrintLevel,
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
        use crate::attributes::field::ArgPrintLevel as P;
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
