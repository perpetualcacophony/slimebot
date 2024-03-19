use poise::{
    serenity_prelude::{CreateActionRow, CreateButton},
    CreateReply,
};

pub trait CreateReplyExt: Default {
    fn new() -> Self {
        Self::default()
    }

    fn button(self, button: CreateButton) -> Self;
}

impl CreateReplyExt for CreateReply {
    fn button(mut self, button: CreateButton) -> Self {
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
