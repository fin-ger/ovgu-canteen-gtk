extern crate gdk;
extern crate gio;
extern crate gio_sys;
extern crate glib;
extern crate glib_sys;
extern crate gtk;
extern crate libc;
extern crate ansi_term;

mod g_action_map;
mod application;
mod widgets;

use std::io::{Write, stderr};
use ansi_term::Colour::*;

fn main()
{
    match application::Application::new()
    {
        Ok(app) => {
            app.run();
        }
        Err(msg) => {
            writeln!(
                &mut stderr(), "\n{}\n\n{}",
                Red.bold().paint("An error occured while running the application:"), msg
            ).unwrap();
            std::process::exit(1);
        },
    };
}
