use attribute_derive::{
    parsing::{AttributeBase, AttributeValue, SpannedValue},
    FromPartial,
};
use syn::spanned::Spanned;

#[derive(Clone, Copy, PartialEq)]
pub struct Level(tracing::Level);

impl quote::ToTokens for Level {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.path().to_tokens(tokens)
    }
}

impl Level {
    pub fn tracing(self) -> tracing::Level {
        self.0
    }

    pub fn path(self) -> syn::Path {
        self.into()
    }
}

impl From<tracing::Level> for Level {
    fn from(value: tracing::Level) -> Self {
        Self(value)
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::ERROR
    }
}

impl From<Level> for syn::Path {
    fn from(level: Level) -> Self {
        let ident = match level {
            Level::ERROR => "ERROR",
            Level::WARN => "WARN",
            Level::INFO => "INFO",
            Level::DEBUG => "DEBUG",
            Level::TRACE => "TRACE",
        };

        syn::parse_str(&format!("::tracing::Level::{ident}")).unwrap()
    }
}

impl Level {
    const ERROR: Self = Self(tracing::Level::ERROR);
    const WARN: Self = Self(tracing::Level::WARN);
    const INFO: Self = Self(tracing::Level::INFO);
    const DEBUG: Self = Self(tracing::Level::DEBUG);
    const TRACE: Self = Self(tracing::Level::TRACE);
}

impl<'a> TryFrom<&'a syn::Ident> for Level {
    type Error = syn::Error;

    fn try_from(ident: &'a syn::Ident) -> Result<Self, Self::Error> {
        match ident.to_string().as_str() {
            "ERROR" => Ok(Self::ERROR),
            "WARN" => Ok(Self::WARN),
            "INFO" => Ok(Self::INFO),
            "DEBUG" => Ok(Self::DEBUG),
            "TRACE" => Ok(Self::TRACE),
            _ => Err(syn::Error::new(ident.span(), "unrecognized level")),
        }
    }
}

impl<'a> TryFrom<&'a syn::Path> for Level {
    type Error = syn::Error;

    fn try_from(path: &'a syn::Path) -> Result<Self, Self::Error> {
        if let Some(ident) = path.get_ident() {
            Self::try_from(ident)
        } else {
            let mut segments = (&path.segments).into_iter();

            if let (Some(tracing), Some(level), Some(ident)) =
                (segments.next(), segments.next(), segments.next())
            {
                if tracing.ident.to_string().as_str() != "tracing" {
                    return Err(syn::Error::new(tracing.span(), "expected 'tracing'"));
                }

                if level.ident.to_string().as_str() != "Level" {
                    return Err(syn::Error::new(tracing.span(), "expected 'Level'"));
                }

                Self::try_from(&ident.ident)
            } else {
                Err(syn::Error::new(path.span(), "expected 3 path segments"))
            }
        }
    }
}

impl AttributeBase for Level {
    type Partial = syn::Path;
}

impl FromPartial<syn::Path> for Level {
    fn from(partial: syn::Path) -> syn::Result<Self> {
        Self::try_from(&partial)
    }
}

impl AttributeValue for Level {
    fn parse_value(input: syn::parse::ParseStream) -> syn::Result<SpannedValue<Self::Partial>> {
        let path: syn::Path = input.parse()?;
        let span = path.span();

        Self::try_from(&path).map(|_level| SpannedValue::new(path, span))
    }
}
