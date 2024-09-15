#[derive(clap::Parser)]
pub struct Cli {
    #[arg(long)]
    env_path: Option<String>,

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
