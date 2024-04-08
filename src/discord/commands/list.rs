use crate::*;

use super::utils::Command;

pub fn list() -> Vec<Command> {
    vec![
        ping(),
        pong(),
        pfp(),
        //watch_fic(),
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
