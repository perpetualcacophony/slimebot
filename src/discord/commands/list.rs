use crate::*;

pub fn list() -> Vec<poise::Command<crate::Data, crate::errors::Error>> {
    vec![
        ping(),
        pong(),
        pfp(),
        watch_fic(),
        echo(),
        ban(),
        banban(),
        uptime(),
        borzoi(),
        cat(),
        fox(),
        minecraft(),
        roll(),
        flip(),
        d20(),
        version(),
        wordle(),
        help(),
        eightball(),
    ]
}
