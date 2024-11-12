use clap::Parser;
// use color_eyre::Result;

mod youdao;
mod cmd;
mod tui;
mod gui;

fn main() {
    let args = cmd::App::parse();

    if let Some(cmd::Command::Gui) = args.command {
        gui::run_gui(args).unwrap()
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                tui::run_tui(args).await
            }).unwrap();
    }
}

