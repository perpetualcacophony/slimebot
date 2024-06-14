use attribute_derive::FromAttr;
use quote::quote;

#[derive(attribute_derive::FromAttr)]
#[attribute(ident = field)]
struct FieldAttribute {
    #[attribute(optional)]
    pub level: tracing::TracingPrintLevel,

    pub rename: Option<String>,
}

#[derive(attribute_derive::FromAttr)]
#[attribute(ident = event)]
struct EventAttribute {
    level: tracing::Level,
}

#[proc_macro_derive(TracingError, attributes(event, field))]
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
    let event = EventAttribute::from_attributes(&input.attrs)?;

    use syn::Data;
    let function_body: Vec<syn::Stmt> = match input.data {
        Data::Struct(data) => {
            let tracing_fields: Vec<tracing::Field> = data
                .fields
                .into_iter()
                .map(tracing::Field::try_from)
                .collect::<Result<_, _>>()?;

            let event = tracing::Event::new(event.level, tracing_fields, true);
            let tracing_event = event.into_macro_call();

            syn::parse_quote! {
                #tracing_event
            }
        }
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|variant| {
                use syn::Fields;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        assert!(fields.unnamed.len() == 1);
                    }
                    _ => unimplemented!(),
                }

                let ident = &variant.ident;

                quote! { #ident(err) => TracingError::event(err) }
            });

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
