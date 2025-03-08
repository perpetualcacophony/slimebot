#[derive(clap::Parser, Debug, Clone)]
pub struct Cli {
    #[arg(long)]
    pub env_path: Option<crate::framework::config::env::Path>,

    #[arg(long)]
    logs: bool,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn logs_enabled(&self) -> bool {
        self.logs || self.command.is_start()
    }

    pub fn notify_on_start(&self) -> bool {
        if let Command::Start(start) = &self.command {
            start.notify()
        } else {
            false
        }
    }
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    Config(Config),
    Start(Start),
}

impl Command {
    pub fn is_start(&self) -> bool {
        matches!(self, Self::Start(..))
    }

    pub fn config(&self) -> Option<&Config> {
        if let Self::Config(ref config) = self {
            Some(config)
        } else {
            None
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
#[group(multiple = false)]
pub struct Config {
    #[arg(long)]
    env: bool,

    #[arg(long)]
    config: bool,
}

impl Config {
    pub fn env(&self) -> bool {
        !self.config
    }

    pub fn config(&self) -> bool {
        !self.env
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            env: true,
            config: true,
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct Start {
    #[arg(long)]
    notify: bool,
}

impl Start {
    pub fn notify(&self) -> bool {
        self.notify
    }
}
