use syn::{spanned::Spanned, Field, Fields};

use proc_macro2::TokenStream;

use quote::ToTokens;

use syn::punctuated::Punctuated;

use crate::{
    attributes::field::TracingPrintLevel,
    tracing_field::{self, TracingField},
};

use syn::Token;

type CommaPunctuated<T> = Punctuated<T, Token![,]>;

pub(crate) struct TracingFields<'a>(CommaPunctuated<TracingField<'a>>);

impl<'a> TracingFields<'a> {
    pub(crate) fn new() -> Self {
        Self(Punctuated::new())
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn push(&mut self, field: TracingField<'a>) {
        self.0.push(field)
    }
}

impl<'a> FromIterator<TracingField<'a>> for TracingFields<'a> {
    fn from_iter<T: IntoIterator<Item = TracingField<'a>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a Field> for TracingFields<'a> {
    fn from_iter<T: IntoIterator<Item = &'a Field>>(iter: T) -> Self {
        let mut peekable = iter.into_iter().peekable();

        if peekable.peek().unwrap().ident.is_some() {
            peekable
                .map(|field| TracingField::try_from(field).expect("all fields have idents"))
                .collect()
        } else {
            peekable
                .enumerate()
                .map(|(index, field)| {
                    TracingField::new_numbered(
                        crate::attributes::Field::try_from(field.attrs.as_slice())
                            .ok()
                            .map(|field| field.print_level().cloned())
                            .flatten()
                            .map(|arg| TracingPrintLevel::from(&arg))
                            .unwrap_or_default(),
                        index,
                        field.span(),
                    )
                })
                .collect()
        }
    }
}

impl ToTokens for TracingFields<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl<'a> From<Punctuated<tracing_field::TracingField<'a>, Token![,]>> for TracingFields<'a> {
    fn from(value: Punctuated<tracing_field::TracingField<'a>, Token![,]>) -> Self {
        Self(value)
    }
}

impl<'a> IntoIterator for TracingFields<'a> {
    type IntoIter =
        <Punctuated<tracing_field::TracingField<'a>, Token![,]> as IntoIterator>::IntoIter;
    type Item = <Punctuated<tracing_field::TracingField<'a>, Token![,]> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a TracingFields<'a> {
    type IntoIter =
        <&'a Punctuated<tracing_field::TracingField<'a>, Token![,]> as IntoIterator>::IntoIter;
    type Item = <&'a Punctuated<tracing_field::TracingField<'a>, Token![,]> as IntoIterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

pub(crate) fn tracing_fields(fields: &Fields) -> TracingFields<'_> {
    match fields {
        Fields::Named(fields) => fields.named.iter().collect(),
        _ => unimplemented!(),
    }
}
