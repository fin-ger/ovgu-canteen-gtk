use anyhow::{Context, Result};
use gdk::Screen;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{ApplicationBuilder, CssProvider};
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};

use crate::components::WindowComponent;

// TODO: add settings window with hamburger menu to access the settings
// ASSIGNEE: @jwuensche

// TODO: add dark theme to settings
// ASSIGNEE: @jwuensche

// TODO: set default canteen in settings
// ASSIGNEE: @jwuensche

// TODO: persist canteen menus on disk for faster loading of app and update menus
//       when loaded
//        - add setting to settings menu for number of menus per canteen to cache
// ASSIGNEE: @fin-ger

// TODO: add reload button for reloading canteen menus on network failure
// ASSIGNEE: @fin-ger

// TODO: Create custom flow widget for menu badges
// ASSIGNEE: ?

// TODO: Replace button with toggle button in menu selection popover to
//       indicate current canteen
// ASSIGNEE: ?

// TODO: create flatpak package

// TODO: write readme

// TODO: try porting to windows metro app

// TODO: try porting to macos app

pub struct Application {
    pub g_app: gtk::Application,
    pub runtime: Runtime,
}

impl Application {
    pub fn new() -> Result<Self> {
        gtk::init().context("Failed to initialize GTK!")?;

        let css_provider = CssProvider::new();
        css_provider
            .load_from_data(std::include_str!("../data/gnome-ovgu-canteen.css").as_bytes())
            .context("Failed to load stylesheets")?;

        let screen = Screen::get_default().context("Cannot find default screen!")?;
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        let runtime = RuntimeBuilder::new()
            .enable_all()
            .threaded_scheduler()
            .thread_name("gnome-ovgu-canteen-tokio")
            .build()
            .context("Cannot create tokio runtime")?;

        let g_app = ApplicationBuilder::new()
            .application_id("org.gnome.ovgu-canteen")
            .build();

        let build_rt = runtime.handle().clone();
        g_app.connect_activate(move |app| match WindowComponent::new(&build_rt, app) {
            Ok(()) => {}
            Err(err) => {
                // TODO: display dialog with error message
                eprintln!("error: {}", err);
                app.quit();
            }
        });

        Ok(Self { g_app, runtime })
    }

    pub fn run(self, args: &[String]) -> i32 {
        self.g_app.run(args)
    }
}
