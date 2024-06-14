use proc_macro2::Span;
use syn::{parse::Parse, spanned::Spanned as _};

#[derive(Clone, PartialEq, Eq, Default)]
pub enum TracingPrintLevel {
    Skip,
    #[default]
    Value,
    Display,
    Debug,
    CustomDebug(syn::LitStr),
}

impl Parse for TracingPrintLevel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        Ok(match ident.to_string().to_lowercase().as_str() {
            "skip" | "none" | "ignore" => Self::Skip,
            "value" => Self::Value,
            "display" => Self::Display,
            "debug" => {
                if let Ok(paren) = input.parse::<syn::ExprParen>() {
                    if let syn::Expr::Lit(expr) = *paren.expr {
                        if let syn::Lit::Str(lit) = expr.lit {
                            return Ok(Self::CustomDebug(lit));
                        }
                    }
                }
                Self::Debug
            }
            _ => return Err(input.error("couldn't recognize print level")),
        })
    }
}

impl TryFrom<syn::Ident> for TracingPrintLevel {
    type Error = syn::Error;

    fn try_from(ident: syn::Ident) -> Result<Self, Self::Error> {
        match ident.to_string().as_str() {
            "skip" | "none" | "ignore" => Ok(Self::Skip),
            "value" => Ok(Self::Value),
            "display" => Ok(Self::Display),
            "debug" => Ok(Self::Debug),
            _ => Err(syn::Error::new(
                ident.span(),
                "couldn't recognize print level",
            )),
        }
    }
}

impl TryFrom<syn::ExprCall> for TracingPrintLevel {
    type Error = syn::Error;

    fn try_from(expr: syn::ExprCall) -> Result<Self, Self::Error> {
        if let syn::Expr::Path(ref expr) = *expr.func {
            if expr.path.require_ident()?.to_string().as_str() != "debug" {
                return Err(syn::Error::new(expr.span(), "ident must be debug"));
            }
        } else {
            return Err(syn::Error::new(expr.span(), "malformed"));
        }

        let argument = expr
            .args
            .first()
            .ok_or(syn::Error::new(expr.span(), "requires 1 arg"))?;
        if expr.args.len() > 1 {
            return Err(syn::Error::new(expr.span(), "exactly 1 arg required"));
        }

        if let syn::Expr::Lit(expr) = argument {
            if let syn::Lit::Str(lit) = &expr.lit {
                Ok(Self::CustomDebug(lit.clone()))
            } else {
                Err(syn::Error::new(expr.span(), "requires string literal"))
            }
        } else {
            Err(syn::Error::new(expr.span(), "requires string literal"))
        }
    }
}

impl TryFrom<syn::Expr> for TracingPrintLevel {
    type Error = syn::Error;

    fn try_from(expr: syn::Expr) -> Result<Self, Self::Error> {
        match expr {
            syn::Expr::Path(expr) => {
                let ident = expr.path.require_ident()?;
                Self::try_from(ident.clone())
            }
            syn::Expr::Call(expr) => Self::try_from(expr),
            _ => Err(syn::Error::new(
                expr.span(),
                "cannot handle this expression",
            )),
        }
    }
}

impl TryFrom<TracingPrintLevel> for syn::Ident {
    type Error = syn::Error;

    fn try_from(value: TracingPrintLevel) -> Result<Self, Self::Error> {
        match value {
            TracingPrintLevel::Debug => Ok(Self::new("debug", Span::call_site())),
            TracingPrintLevel::Value => Ok(Self::new("value", Span::call_site())),
            TracingPrintLevel::Display => Ok(Self::new("display", Span::call_site())),
            TracingPrintLevel::Skip => Ok(Self::new("skip", Span::call_site())),
            _ => Err(syn::Error::new(Span::call_site(), "not an identifier")),
        }
    }
}

impl From<TracingPrintLevel> for syn::Expr {
    fn from(value: TracingPrintLevel) -> Self {
        if let Ok(ident) = syn::Ident::try_from(value.clone()) {
            syn::parse_quote! { #ident }
        } else if let TracingPrintLevel::CustomDebug(lit) = value {
            Self::Call(syn::parse_quote! { debug!(#lit) })
        } else {
            unreachable!()
        }
    }
}

impl attribute_derive::parsing::AttributeValue for TracingPrintLevel {
    fn parse_value(
        input: syn::parse::ParseStream,
    ) -> syn::Result<attribute_derive::parsing::SpannedValue<Self::Partial>> {
        Self::parse(input)
            .map(|value| attribute_derive::parsing::SpannedValue::new(value.into(), input.span()))
    }
}

impl attribute_derive::parsing::AttributeBase for TracingPrintLevel {
    type Partial = syn::Expr;
}

impl attribute_derive::FromPartial<syn::Expr> for TracingPrintLevel {
    fn from(expr: syn::Expr) -> syn::Result<Self> {
        Self::try_from(expr)
    }
}
