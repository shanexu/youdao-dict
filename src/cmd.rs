use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "yd", version)]
pub struct App {
    #[clap(flatten)]
    pub global_opts: GlobalOpts,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Tui,
    Gui,
}

#[derive(Debug, Args)]
pub struct GlobalOpts {
    #[arg(short, long, global = true )]
    pub word: Option<String>,
}
