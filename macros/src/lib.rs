use attribute_derive::FromAttr;
use quote::quote;

#[derive(attribute_derive::FromAttr)]
#[attribute(ident = field)]
struct FieldAttribute {
    #[attribute(optional)]
    pub print: tracing::TracingPrintLevel,

    pub rename: Option<String>,
}

#[derive(attribute_derive::FromAttr)]
#[attribute(ident = event)]
struct EventAttribute {
    #[attribute(optional)]
    level: tracing::Level,
}

#[derive(attribute_derive::FromAttr)]
#[attribute(ident = span)]
struct SpanAttribute {
    #[attribute(optional)]
    level: tracing::Level,
}

#[proc_macro_derive(TracingError, attributes(event, field, span))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let expanded = expand(input);
    expanded
        .map(quote::ToTokens::into_token_stream)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

mod tracing;

fn expand(input: syn::DeriveInput) -> syn::Result<syn::ItemImpl> {
    use syn::Data;
    let function_body: Vec<syn::Stmt> = match input.data {
        Data::Struct(data) => {
            let attr = EventAttribute::from_attributes(&input.attrs)?;

            let tracing_fields: Vec<tracing::Field> = data
                .fields
                .into_iter()
                .map(tracing::Field::try_from)
                .collect::<Result<_, _>>()?;

            let event = tracing::Event::new(attr.level, tracing_fields, true);
            let tracing_event = event.into_macro_call();

            syn::parse_quote! {
                #tracing_event
            }
        }
        Data::Enum(data) => {
            use heck::ToSnakeCase;

            let span = SpanAttribute::from_attributes(&input.attrs)?;
            let level = span.level;

            let variants = data
                .variants
                .iter()
                .map(|variant| {
                    let span_name = variant.ident.to_string().to_snake_case();

                    use syn::Fields;
                    match &variant.fields {
                        Fields::Unnamed(fields) => {
                            assert!(fields.unnamed.len() == 1);
                        }
                        _ => unimplemented!(),
                    }

                    let ident = &variant.ident;

                    let match_return = if let Some(attr) = variant
                        .attrs
                        .iter()
                        .find(|attr| attr.path().is_ident("event"))
                    {
                        let attr = EventAttribute::from_attribute(attr)?;
                        let event = tracing::Event::new_custom(
                            attr.level,
                            Vec::default(),
                            syn::parse_str("err").unwrap(),
                        );
                        let tracing_event = event.into_macro_call();

                        quote! { #tracing_event }
                    } else {
                        quote! { TracingError::event(err) }
                    };

                    Ok(quote! {
                        #ident(err) => {
                            let span = ::tracing::span!(#level, #span_name);
                            let _enter = span.enter();

                            #match_return
                        }
                    })
                })
                .collect::<Result<Vec<_>, syn::Error>>()?;
            syn::parse_quote! {
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

    Ok(syn::parse_quote! {
        impl TracingError for #name {
            fn event(&self) {
                #(#function_body)*
            }
        }
    })
}
