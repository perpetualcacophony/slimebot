use proc_macro2::TokenStream;

use quote::{quote_spanned, ToTokens, TokenStreamExt};

use crate::rename_from_attrs;

use super::display_mode_from_attrs;

use syn::{spanned::Spanned, LitStr, Token};

use proc_macro2::Span;

use syn::Expr;

use syn::Ident;

use std::borrow::Cow;

use super::TracingDisplayMode;

pub(crate) struct TracingField<'i> {
    pub(crate) display_mode: TracingDisplayMode,
    pub(crate) name: Cow<'i, Ident>,
    rename: Option<LitStr>,
    pub(crate) value: Expr,
    pub(crate) span: Span,
}

impl<'i> TracingField<'i> {
    pub(crate) fn new(
        display_mode: TracingDisplayMode,
        name: &'i Ident,
        rename: Option<LitStr>,
        span: Span,
    ) -> Self {
        Self::new_cow(display_mode, Cow::Borrowed(name), rename, span)
    }

    pub(crate) fn new_cow(
        display_mode: TracingDisplayMode,
        name: Cow<'i, Ident>,
        rename: Option<LitStr>,
        span: Span,
    ) -> Self {
        Self {
            display_mode,
            name: name.clone(),
            rename,
            value: Expr::Field(syn::ExprField {
                attrs: Vec::new(),
                base: Box::new(Expr::Path(syn::ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: syn::parse_str("self").unwrap(),
                })),
                dot_token: <Token![.]>::default(),
                member: syn::Member::Named(name.into_owned()),
            }),
            span,
        }
    }

    pub(crate) fn new_numbered(display_mode: TracingDisplayMode, index: usize, span: Span) -> Self {
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
        if self.display_mode != TracingDisplayMode::Skip {
            let name = self
                .rename
                .as_ref()
                .map_or_else(|| self.name().to_string(), |lit| lit.value());
            let display_mode = self.display_mode;
            let value = &self.value;

            let expanded = quote_spanned! { self.span=>
                #name
                =
                #display_mode
                #value
            };
            tokens.append_all(expanded);
        }
    }
}
