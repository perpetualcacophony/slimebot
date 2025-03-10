#[derive(Debug, Copy, Clone)]
pub struct GameOptions {
    pub style: GameStyle,
    pub guesses_limit: Option<usize>,
}

#[allow(clippy::derivable_impls)]
impl Default for GameOptions {
    fn default() -> Self {
        Self {
            style: GameStyle::default(),
            guesses_limit: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct GameOptionsBuilder {
    style: Option<GameStyle>,
    guesses_limit: Option<usize>,
}

impl GameOptionsBuilder {
    pub fn style(mut self, style: impl Into<Option<GameStyle>>) -> Self {
        self.style = style.into();
        self
    }

    pub fn _guesses_limit(mut self, limit: impl Into<Option<usize>>) -> Self {
        self.guesses_limit = limit.into();
        self
    }

    pub fn build(self) -> GameOptions {
        GameOptions {
            style: self.style.unwrap_or_default(),
            guesses_limit: self.guesses_limit,
        }
    }
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
