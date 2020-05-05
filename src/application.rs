use anyhow::{Context, Result};
use gdk::Screen;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{
    ApplicationBuilder, ButtonsType, CssProvider, MessageDialogBuilder, MessageType, WindowPosition,
};
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};

use crate::components::WindowComponent;

pub struct Application {
    pub g_app: gtk::Application,
    pub runtime: Runtime,
}

impl Application {
    pub fn new() -> Result<Self> {
        log::debug!("initializing ovgu-canteen-gtk");
        gtk::init().context("Failed to initialize GTK!")?;

        let css_provider = CssProvider::new();
        css_provider
            .load_from_data(std::include_str!("../data/de.fin_ger.OvGUCanteen.css").as_bytes())
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
            .thread_name("ovgu-canteen-gtk-tokio")
            .build()
            .context("Cannot create tokio runtime")?;

        let g_app = ApplicationBuilder::new()
            .application_id("de.fin_ger.OvGUCanteen")
            .build();

        let build_rt = runtime.handle().clone();
        g_app.connect_activate(move |app| match WindowComponent::new(&build_rt, app) {
            Ok(()) => {}
            Err(err) => {
                log::error!("error starting application: {:#}", err);
                let dialog = MessageDialogBuilder::new()
                    .buttons(ButtonsType::Close)
                    .message_type(MessageType::Error)
                    .text("Error starting the application!")
                    .secondary_text(&format!("<tt>{:#}</tt>", err))
                    .secondary_use_markup(true)
                    .application(app)
                    .destroy_with_parent(true)
                    .icon_name("dialog-error")
                    .resizable(true)
                    .window_position(WindowPosition::Center)
                    .build();
                let children = dialog
                    .get_message_area()
                    .unwrap()
                    .downcast::<gtk::Container>()
                    .unwrap()
                    .get_children();
                for child in children {
                    if child.is::<gtk::Label>() {
                        child.downcast::<gtk::Label>().unwrap().set_selectable(true);
                    }
                }
                dialog.run();
                log::debug!("quitting ovgu-canteen-gtk");
                app.quit();
            }
        });

        log::debug!("finish initializing ovgu-canteen-gtk");

        Ok(Self { g_app, runtime })
    }

    pub fn run(self, args: &[String]) -> i32 {
        log::debug!("running ovgu-canteen-gtk");
        self.g_app.run(args)
    }
}
