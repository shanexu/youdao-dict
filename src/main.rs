use clap::Parser;
// use color_eyre::Result;

mod youdao;
mod cmd;
mod tui;
mod tabs;
mod db;
mod models;
mod schema;

fn main() {
    let args = cmd::App::parse();

    let mut conn = db::establish_connection();

    db::list_history(&mut conn);
    db::create_history(&mut conn, "world");

    if let Some(cmd::Command::Gui) = args.command {
        // gui::run_gui(args).unwrap()
        tabs::main::run_tabs(args).unwrap()
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

