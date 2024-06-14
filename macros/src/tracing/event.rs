use proc_macro2::TokenStream;
use quote::quote;

use crate::tracing;

type CommaPunctuared<T> = syn::punctuated::Punctuated<T, syn::Token![,]>;

/// The contents of a tracing `event!` macro call, with a verbosity level, optional fields, and a display string.
///
/// Example: `tracing::Level::ERROR, field = 123, field_debug = ?vec![4, 5, 6]`
pub struct Event<'a> {
    /// The path to the relevant tracing [Level](::tracing::Level).
    level: tracing::Level,

    /// The event's associated [tracing::Fields].
    fields: Vec<tracing::Field<'a>>,

    /// Whether or not to include the error's [Display] implementation in the event's message.
    display: bool,
}

impl<'a> Event<'a> {
    /// Constructs a new Event.
    pub fn new(level: tracing::Level, fields: Vec<tracing::Field<'a>>, display: bool) -> Self {
        Self {
            level,
            fields,
            display,
        }
    }

    pub fn into_macro_call(self) -> Macro<'a> {
        Macro(self)
    }
}

impl quote::ToTokens for Event<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.level.to_tokens(tokens);

        if !self.fields.is_empty() || self.display {
            <syn::Token![,]>::default().to_tokens(tokens);
        }

        if !self.fields.is_empty() {
            let punctuated = CommaPunctuared::from_iter(self.fields.iter());
            punctuated.to_tokens(tokens);

            if self.display {
                <syn::Token![,]>::default().to_tokens(tokens);
            }
        }

        if self.display {
            quote! { "{}", self }.to_tokens(tokens);
        }
    }
}

/// Represents an entire tracing `event!` macro call, wrapping an [Event].
///
/// Constructed by [Event::into_macro_call].
pub struct Macro<'a>(Event<'a>);

/// Wraps the inner [Event]'s ToTokens implementation with `tracing::event!( /* ... */`
impl quote::ToTokens for Macro<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let event = &self.0;
        quote! { tracing::event!(#event) }.to_tokens(tokens)
    }
}
