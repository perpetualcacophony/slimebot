#[derive(poise::ChoiceParameter, Debug, Clone, Copy, PartialEq)]
pub enum GameType {
    #[name = "daily"]
    Daily,
    #[name = "random"]
    Random,
}

#[derive(poise::ChoiceParameter, Debug, Clone, Copy, Default)]
pub enum GameStyle {
    #[name = "colors only"]
    #[name = "colors"]
    #[name = "colors_only"]
    #[name = "hidden"]
    Colors,
    #[name = "with letters"]
    #[name = "letters"]
    #[name = "with_letters"]
    #[name = "anx"]
    #[default]
    Letters,
    #[name = "spaced letters"]
    #[name = "spaced_letters"]
    #[name = "spaced"]
    #[name = "with spaces"]
    #[name = "with_spaces"]
    #[name = "letters with spaces"]
    #[name = "letters_with_spaces"]
    #[name = "fix flags"]
    #[name = "fix_flags"]
    SpacedLetters,
}

impl GameStyle {
    pub fn parse(style: Option<Self>, fix_flags: bool) -> Self {
        if fix_flags {
            Self::SpacedLetters
        } else {
            style.unwrap_or_default()
        }
    }
}
