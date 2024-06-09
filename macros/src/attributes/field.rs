use proc_macro2::Span;
use syn::{
    parse::{discouraged::Speculative, Parse},
    Token,
};

type CommaPunctuated<T> = syn::punctuated::Punctuated<T, Token![,]>;

pub struct MetaList {
    pub args: CommaPunctuated<Argument>,
}

impl MetaList {
    pub fn print_level(&self) -> Option<&ArgPrintLevel> {
        self.args.iter().filter_map(Argument::print_level).next()
    }

    pub fn rename(&self) -> Option<&ArgRename> {
        self.args.iter().filter_map(Argument::rename).next()
    }
}

impl std::ops::Index<usize> for MetaList {
    type Output = Argument;

    fn index(&self, index: usize) -> &Self::Output {
        self.args.index(index)
    }
}

impl Parse for MetaList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            args: CommaPunctuated::parse_terminated(input)?,
        })
    }
}

pub enum Argument {
    PrintLevel(ArgPrintLevel),
    Rename(ArgRename),
}

impl Argument {
    fn print_level(&self) -> Option<&ArgPrintLevel> {
        if let Self::PrintLevel(ref arg) = self {
            Some(arg)
        } else {
            None
        }
    }

    fn rename(&self) -> Option<&ArgRename> {
        if let Self::Rename(ref arg) = self {
            Some(arg)
        } else {
            None
        }
    }
}

impl Parse for Argument {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        if let Ok(arg) = fork.parse::<ArgPrintLevel>() {
            input.advance_to(&fork);
            return Ok(Self::PrintLevel(arg));
        }

        let fork = input.fork();
        if let Ok(arg) = fork.parse::<ArgRename>() {
            input.advance_to(&fork);
            return Ok(Self::Rename(arg));
        }

        Err(input.error("couldn't recognize an argument!"))
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum ArgPrintLevel {
    Skip,
    Value,
    Display,
    Debug,
    CustomDebug(syn::LitStr),
}

impl ArgPrintLevel {
    fn ident() -> syn::Ident {
        syn::Ident::new("print", Span::call_site())
    }
}

impl Parse for ArgPrintLevel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<syn::Ident>()? != Self::ident() {
            return Err(input.error("wrong identifier!"));
        }

        input.parse::<Token![=]>()?;

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

pub struct ArgRename {
    pub lit: syn::LitStr,
}

impl ArgRename {
    fn ident() -> syn::Ident {
        syn::Ident::new("rename", Span::call_site())
    }
}

impl Parse for ArgRename {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<syn::Ident>()? != Self::ident() {
            return Err(input.error("wrong identifier!"));
        }

        input.parse::<Token![=]>()?;

        Ok(Self {
            lit: input.parse()?,
        })
    }
}
