macro_rules! list {
    ($($pub:vis $module:ident$(: {$($cmd:ident),+})?),+) => {
        $(
            $pub mod $module;
            use $module::$module;

            $(
                $(
                    #[allow(unused_imports)]
                    use $module::$cmd;
                )+
            )?
        )+

        pub fn list() -> Vec<crate::utils::poise::Command> {
            let mut vec = vec![
                $($module()),+
            ];

            #[cfg(feature = "wordle")]
            vec.push(wordle());

            vec
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
    pub minecraft,
    roll: {d20},
    flip,
    version,
    help,
    eightball,
    januannie
}

#[cfg(feature = "wordle")]
pub mod wordle;

#[cfg(feature = "wordle")]
use wordle::wordle;

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
