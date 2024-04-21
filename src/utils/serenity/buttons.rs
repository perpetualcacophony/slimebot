use poise::{
    serenity_prelude::{
        CreateActionRow, CreateButton, CreateInteractionResponseMessage, CreateMessage,
        ReactionType,
    },
    CreateReply,
};

pub trait AddButton: Sized + Clone {
    fn add_button(mut self, button: CreateButton) -> Self {
        self.add_button_in_place(button);
        self
    }

    fn add_button_in_place(&mut self, button: CreateButton) {
        let cloned = self.clone();
        *self = cloned.add_button(button);
    }

    fn add_buttons(mut self, buttons: &[CreateButton]) -> Self {
        for button in buttons {
            self = self.add_button(button.clone());
        }

        self
    }

    #[allow(dead_code)]
    fn add_buttons_in_place(&mut self, buttons: &[CreateButton]) {
        for button in buttons {
            self.add_button_in_place(button.clone());
        }
    }
}

impl AddButton for CreateInteractionResponseMessage {
    fn add_button(self, button: CreateButton) -> Self {
        self.button(button)
    }
}

impl AddButton for CreateMessage {
    fn add_button(self, button: CreateButton) -> Self {
        self.button(button)
    }
}

impl AddButton for CreateReply {
    fn add_button(mut self, button: CreateButton) -> Self {
        if let Some(ref mut rows) = self.components {
            if let Some(buttons) = rows.iter_mut().find_map(|row| match row {
                CreateActionRow::Buttons(b) => Some(b),
                _ => None,
            }) {
                buttons.push(button);
            } else {
                rows.push(CreateActionRow::Buttons(vec![button]));
            }
        } else {
            self = self.components(vec![CreateActionRow::Buttons(vec![button])]);
        }

        self
    }
}

pub trait YesNoButtons: AddButton {
    fn yes_no_buttons(self) -> Self {
        let yes_emoji = ReactionType::Unicode("✅".to_owned());
        let no_emoji = ReactionType::Unicode("❌".to_owned());

        let yes_button = CreateButton::new("yes")
            .emoji(yes_emoji)
            .label("yes")
            .style(poise::serenity_prelude::ButtonStyle::Secondary);

        let no_button = CreateButton::new("no")
            .emoji(no_emoji)
            .label("no")
            .style(poise::serenity_prelude::ButtonStyle::Secondary);

        self.add_buttons(&[yes_button, no_button])
    }
}

impl<T> YesNoButtons for T where T: AddButton {}
