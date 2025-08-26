mod parser;
mod app;

fn main() {
    let terminal = ratatui::init();
    match app::App::new().run(terminal) {
        Ok(_) => {},
        Err(e) => panic!("got error {}", e)
    }
    ratatui::restore();
}
