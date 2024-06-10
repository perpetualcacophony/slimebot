use proc_macro2::Span;
use syn::{
    parse::{discouraged::Speculative, Parse},
    Token,
};

type CommaPunctuated<T> = syn::punctuated::Punctuated<T, Token![,]>;

pub struct MetaList {
    args: CommaPunctuated<Argument>,
}

impl MetaList {
    pub fn level(&self) -> Option<&ArgLevel> {
        self.args.iter().filter_map(Argument::level).next()
    }
}

impl Parse for MetaList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            args: CommaPunctuated::parse_terminated(input)?,
        })
    }
}

impl std::ops::Index<usize> for MetaList {
    type Output = Argument;

    fn index(&self, index: usize) -> &Self::Output {
        self.args.index(index)
    }
}

pub enum Argument {
    Level(ArgLevel),
}

impl Argument {
    pub fn level(&self) -> Option<&ArgLevel> {
        match self {
            Self::Level(ref level) => Some(level),
            _ => None,
        }
    }
}

impl Parse for Argument {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        if let Ok(level) = fork.parse::<ArgLevel>() {
            input.advance_to(&fork);
            return Ok(Self::Level(level));
        }

        Err(input.error("couldn't recognize an argument!"))
    }
}

pub struct ArgLevel {
    pub level: syn::Path,
}

impl ArgLevel {
    fn ident() -> syn::Ident {
        syn::Ident::new("level", Span::call_site())
    }
}

// level = <Path>
impl Parse for ArgLevel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.parse::<syn::Ident>()? != Self::ident() {
            return Err(input.error("wrong identifier!"));
        }

        input.parse::<Token![=]>()?;

        let path: syn::Path = input.parse()?;
        let level = if path.segments.len() == 1 {
            let mut new_path: syn::Path = syn::parse_str("tracing::Level").unwrap();
            new_path.segments.push(path.segments[0].clone());
            new_path
        } else {
            path
        };

        Ok(Self { level })
    }
}
