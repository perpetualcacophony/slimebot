macro_rules! list {
    ($($module:ident{$($cmd:ident),*$($feature:literal)?})+) => {
        $(
            $(
                #[cfg(feature = $feature)]
            )?
            pub mod $module;
        )+

        #[allow(clippy::vec_init_then_push)]
        pub fn list() -> Vec<crate::utils::poise::Command> {
            let mut vec = Vec::with_capacity(${count($module)});

            $(
                $(
                    #[cfg(feature = $feature)]
                )?
                vec.push($module::$module());

                $(
                    vec.push($module::$cmd());
                )*
            )+

            vec
        }
    };
}

list! {
    ping{pong}
    pfp{}
    ban{}
    banban{}
    uptime{}
    borzoi{}
    cat{}
    fox{}
    minecraft{}
    roll{d20}
    flip{}
    version{}
    help{}
    eightball{}
    januannie{}
    wordle{"wordle"}
    nortverse{"nortverse"}
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
