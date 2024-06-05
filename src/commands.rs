macro_rules! list {
    ($($pub:vis $module:ident$(: {$($cmd:ident),+})?),+) => {
        $(
            $pub mod $module;
            use $module::$module;

            $(
                $(
                    use $module::$cmd;
                )+
            )?
        )+

        pub fn list() -> Vec<crate::utils::poise::Command> {

            vec![
                $($module()),+
            ]
        }
    };
}

list! {
    ping: {pong},
    pfp,
    ban,
    banban,
    uptime,
    borzoi,
    cat,
    fox,
    minecraft,
    roll: {d20},
    flip,
    version,
    pub wordle,
    help,
    eightball
}

trait LogCommands {
    async fn log_command(&self);
}

impl LogCommands for crate::utils::Context<'_> {
    async fn log_command(&self) {
        let channel = self
            .channel_id()
            .name(self.http())
            .await
            .map_or("dms".to_string(), |c| format!("#{c}"));
        tracing::info!(
            "@{} ({}): {}",
            self.author().name,
            channel,
            self.invocation_string()
        );
    }
}
