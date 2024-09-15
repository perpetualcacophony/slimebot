#[derive(clap::Parser)]
pub struct Cli {
    #[arg(long)]
    pub env_path: Option<crate::framework::config::env::Path>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Default)]
pub enum Command {
    Config,

    #[default]
    Start,
}

impl Command {
    pub fn is_start(&self) -> bool {
        matches!(self, Self::Start)
    }
}
