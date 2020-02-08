mod application;
mod components;

fn main() {
    match application::Application::new() {
        Ok(app) => {
            std::process::exit(app.run(std::env::args().collect()));
        }
        Err(msg) => {
            eprintln!("error: {}", msg);
            std::process::exit(1337);
        }
    };
}
