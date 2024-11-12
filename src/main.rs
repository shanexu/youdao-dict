use clap::Parser;
use color_eyre::Result;

mod youdao;
mod cmd;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cmd::App::parse();
    println!("{:?}", args);

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = tui::App::new(args.global_opts.word.unwrap_or_default());
    let app_result = app.run(terminal).await;
    ratatui::restore();
    app_result
}
