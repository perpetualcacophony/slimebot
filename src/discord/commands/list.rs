macro_rules! list {
    ($($command:ident),+) => {
        pub fn list() -> Vec<crate::utils::poise::Command> {
            vec![
                $(super::$command()),+
            ]
        }
    };
}

list! {
    ping,
    pong,
    pfp,
    echo,
    ban,
    banban,
    uptime,
    borzoi,
    cat,
    fox,
    minecraft,
    roll,
    flip,
    d20,
    version,
    wordle,
    help,
    eightball
}
