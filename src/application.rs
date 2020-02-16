use gdk::Screen;
use gio::prelude::*;
use gio::ApplicationFlags;
use gtk::prelude::*;
use gtk::{AboutDialog, Builder, Button, CssProvider};
use ovgu_canteen::{Canteen, CanteenDescription};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::{Builder as RuntimeBuilder, Handle, Runtime};
use tokio::sync::mpsc::channel;

use crate::components::{CanteenComponent, WindowComponent, GLADE};

// TODO: set offset of canteen popup-menu so that the current item is on the
//       mouse position
// ASSIGNEE: @fin-ger

// TODO: show authors from Cargo.toml in about dialog
// ASSIGNEE: @fin-ger

// TODO: add settings window with hamburger menu to access the settings
// ASSIGNEE: @jwuensche

// TODO: move about button to hamburger menu
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

fn build(rt: &Handle, app: &gtk::Application) -> Result<(), &'static str> {
    let builder = Builder::new_from_string(GLADE);

    let window = WindowComponent {
        window: builder.get_object("window").unwrap(),
        canteens_stack: Rc::new(RefCell::new(builder.get_object("canteens-stack").unwrap())),
        canteen_label: Rc::new(RefCell::new(builder.get_object("canteen-label").unwrap())),
        canteens_menu: builder.get_object("canteens-menu").unwrap(),
    };
    let about_dialog: AboutDialog = builder.get_object("about").unwrap();
    let about_button: Button = builder.get_object("about-btn").unwrap();

    about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
    about_button.connect_clicked(move |_btn| {
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
        canteen_components.push(CanteenComponent::new(canteen_desc.clone(), &window));
        let mut canteen_tx = tx.clone();
        rt.spawn(async move {
            let canteen = Canteen::new(canteen_desc.clone()).await;
            canteen_tx.send((canteen_desc, canteen)).await.unwrap();
        });
    }

    let c = glib::MainContext::default();
    c.spawn_local(async move {
        // fetching parallel loaded canteens here and inserting
        // one canteen after another into the GUI.
        while let Some((desc, canteen)) = rx.recv().await {
            let comp = canteen_components
                .iter()
                .find(|c| c.description == desc)
                .unwrap();
            comp.loaded(canteen).await;
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
    pub fn new() -> Result<Self, &'static str> {
        gtk::init().map_err(|_| "Failed to initialize GTK!")?;

        let css_provider = CssProvider::new();
        css_provider
            .load_from_data(std::include_str!("../data/gnome-ovgu-canteen.css").as_bytes())
            .unwrap();

        let screen = Screen::get_default().ok_or("Cannot find default screen!")?;
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
            .unwrap();

        let g_app =
            gtk::Application::new(Some("org.gnome.ovgu-canteen"), ApplicationFlags::default())
                .map_err(|_| "Failed to create application!")?;

        let build_rt = runtime.handle().clone();
        g_app.connect_activate(move |app| match build(&build_rt, app) {
            Ok(()) => {}
            Err(err) => {
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
