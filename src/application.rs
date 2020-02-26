use anyhow::{bail, Context, Result};
use cargo_author::Author;
use gdk::Screen;
use gdk_pixbuf::PixbufLoader;
use gio::prelude::*;
use gdk_pixbuf::prelude::*;
use gtk::prelude::*;
use gtk::{ApplicationBuilder, AboutDialog, Builder, Button, CssProvider, MenuButton};
use ovgu_canteen::{Canteen, CanteenDescription};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::{Builder as RuntimeBuilder, Handle, Runtime};
use tokio::sync::mpsc::channel;

use crate::components::{get, CanteenComponent, WindowComponent, GLADE, ICON};

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
// ASSIGNEE: @jwuensche

// TODO: create flatpak package

// TODO: write readme

// TODO: try porting to windows metro app

// TODO: try porting to macos app

fn build(rt: &Handle, app: &gtk::Application) -> Result<()> {
    let builder = Builder::new_from_string(GLADE);

    let window = WindowComponent {
        window: get(&builder, "window")?,
        canteens_stack: Rc::new(RefCell::new(get(&builder, "canteens-stack")?)),
        canteen_label: Rc::new(RefCell::new(get(&builder, "canteen-label")?)),
        canteen_menu_button: Rc::new(RefCell::new(get(&builder, "canteen-menu-button")?)),
        canteens_menu_box: get(&builder, "canteens-menu-box")?,
    };
    let about_dialog: AboutDialog = get(&builder, "about")?;
    let about_button: Button = get(&builder, "about-btn")?;
    let options_button: MenuButton = get(&builder, "options-button")?;

    let icon_loader = PixbufLoader::new();
    icon_loader.write(ICON.as_bytes()).context("Failed to create icon")?;
    icon_loader.close().context("Failed to create icon")?;
    let icon = icon_loader.get_pixbuf().context("Failed to create icon")?;
    window.window.set_icon(Some(&icon));
    about_dialog.set_logo(Some(&icon));

    let authors = env!("CARGO_PKG_AUTHORS")
        .split(':')
        .map(|author| Author::new(author))
        .collect::<Vec<_>>();

    about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
    about_dialog.set_website(Some(env!("CARGO_PKG_REPOSITORY")));
    about_dialog.set_website_label(Some("Source Code"));
    about_dialog.set_comments(Some(env!("CARGO_PKG_DESCRIPTION")));
    about_dialog.set_authors(
        &authors
            .iter()
            .map(|author| {
                if let Some(name) = &author.name {
                    Ok(name.as_str())
                } else if let Some(email) = &author.email {
                    Ok(email.as_str())
                } else if let Some(url) = &author.url {
                    Ok(url.as_str())
                } else {
                    bail!("Failed to get author name");
                }
            })
            .collect::<Result<Vec<_>>>()?,
    );
    about_button.connect_clicked(move |_btn| {
        if let Some(popover) = options_button.get_popover() {
            popover.popdown();
        }

        about_dialog.run();
        about_dialog.hide();
    });

    let mut canteens = vec![
        CanteenDescription::UniCampusLowerHall,
        CanteenDescription::UniCampusUpperHall,
        CanteenDescription::Kellercafe,
        CanteenDescription::Herrenkrug,
        CanteenDescription::Stendal,
        CanteenDescription::Wernigerode,
        CanteenDescription::DomCafeteHalberstadt,
    ];

    // canteens are downloaded in parallel here,
    // but in order for one canteen to show up in a batch
    // we are using an mpsc channel to put the parallel loaded canteens
    // in an order which is later sequentially inserted into the GUI.
    let (tx, mut rx) = channel(canteens.len());
    let mut canteen_components = Vec::new();
    for canteen_desc in canteens.drain(..) {
        let comp = CanteenComponent::new(canteen_desc.clone(), &window)
            .context(format!("Failed to create canteen {:?}", canteen_desc))?;
        canteen_components.push(comp);
        let mut canteen_tx = tx.clone();
        rt.spawn(async move {
            let canteen = Canteen::new(canteen_desc.clone()).await;
            if let Err(e) = canteen_tx.send((canteen_desc, canteen)).await {
                eprintln!("error: {}", e);
                // TODO: handle tx send error by displaying canteen not available
            }
        });
    }

    let c = glib::MainContext::default();
    c.spawn_local(async move {
        // fetching parallel loaded canteens here and inserting
        // one canteen after another into the GUI.
        // TODO: render currently visible canteen first
        while let Some((desc, canteen)) = rx.recv().await {
            if let Some(comp) = canteen_components.iter().find(|c| c.description == desc) {
                comp.loaded(canteen).await;
            } else {
                eprintln!("canteen {:?} not found in components list", desc);
                // TODO: display error dialog
            }
        }
    });

    window.window.set_application(Some(app));
    window.window.show_all();

    Ok(())
}

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
        g_app.connect_activate(move |app| match build(&build_rt, app) {
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
