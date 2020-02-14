use gtk::{Builder, CssProvider, Button, AboutDialog};
use gtk::prelude::*;
use gio::prelude::*;
use gdk::{Screen};
use ovgu_canteen::{Canteen, CanteenDescription};
use tokio::runtime::{Builder as RuntimeBuilder, Runtime, Handle};
use tokio::sync::oneshot::channel;

use crate::glib_yield;
use crate::components::{GLADE, WindowComponentBuilder, DayComponent};

// TODO: handle error when canteen menu failed to load by displaying error message

// TODO: show loading indicator while a canteen menu is fetched:
//        - when no data for the canteen is available show loading message in
//          canteens stack page
//        - animate the reload button

// TODO: persist canteen menus on disk for faster loading of app and update menus
//       when loaded

// TODO: add settings window with hamburger menu to access the settings

// TODO: add dark theme to settings

// TODO: set default canteen in settings

// TODO: move about button to hamburger menu

// TODO: add reload button for reloading canteen menus on network failure

// TODO: set offset of canteen popup-menu so that the current item is on the
//       mouse position

// TODO: show authors from Cargo.toml in about dialog

// TODO: create flatpak package

// TODO: write readme

// TODO: try porting to windows metro app

// TODO: try porting to macos app

fn build(rt: &Handle, app: &gtk::Application) -> Result<(), &'static str> {
    let builder = Builder::new_from_string(GLADE);

    let window = WindowComponentBuilder {
        window: builder.get_object("window").unwrap(),
        canteen_stack: builder.get_object("canteen-stack").unwrap(),
        canteen_label: builder.get_object("canteen-label").unwrap(),
        lower_hall_days_box: builder.get_object("lower-hall-days-box").unwrap(),
        upper_hall_days_box: builder.get_object("upper-hall-days-box").unwrap(),
        kellercafe_days_box: builder.get_object("kellercafe-days-box").unwrap(),
        herrenkrug_days_box: builder.get_object("herrenkrug-days-box").unwrap(),
        stendal_days_box: builder.get_object("stendal-days-box").unwrap(),
        wernigerode_days_box: builder.get_object("wernigerode-days-box").unwrap(),
        dom_cafete_days_box: builder.get_object("dom-cafete-days-box").unwrap(),
        lower_hall_item: builder.get_object("ovgu-lower-hall").unwrap(),
        upper_hall_item: builder.get_object("ovgu-upper-hall").unwrap(),
        kellercafe_item: builder.get_object("kellercafe").unwrap(),
        herrenkrug_item: builder.get_object("herrenkrug").unwrap(),
        stendal_item: builder.get_object("stendal").unwrap(),
        wernigerode_item: builder.get_object("wernigerode").unwrap(),
        dom_cafete_item: builder.get_object("dom-cafete").unwrap(),
    }.build();
    let about_dialog: AboutDialog = builder.get_object("about").unwrap();
    let about_button: Button = builder.get_object("about-btn").unwrap();

    about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
    about_button.connect_clicked(move |_btn| {
        about_dialog.run();
        about_dialog.hide();
    });

    let lower_hall_stack_handle = window.canteen_stack.clone();
    let lower_hall_label_handle = window.canteen_label.clone();
    window.lower_hall_item.borrow_mut().connect_activate(move |item| {
        lower_hall_stack_handle.borrow()
            .set_visible_child_name("ovgu-lower-hall");
        lower_hall_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let upper_hall_stack_handle = window.canteen_stack.clone();
    let upper_hall_label_handle = window.canteen_label.clone();
    window.upper_hall_item.borrow_mut().connect_activate(move |item| {
        upper_hall_stack_handle.borrow()
            .set_visible_child_name("ovgu-upper-hall");
        upper_hall_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let kellercafe_stack_handle = window.canteen_stack.clone();
    let kellercafe_label_handle = window.canteen_label.clone();
    window.kellercafe_item.borrow_mut().connect_activate(move |item| {
        kellercafe_stack_handle.borrow()
            .set_visible_child_name("kellercafe");
        kellercafe_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let herrenkrug_stack_handle = window.canteen_stack.clone();
    let herrenkrug_label_handle = window.canteen_label.clone();
    window.herrenkrug_item.borrow_mut().connect_activate(move |item| {
        herrenkrug_stack_handle.borrow()
            .set_visible_child_name("herrenkrug");
        herrenkrug_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let stendal_stack_handle = window.canteen_stack.clone();
    let stendal_label_handle = window.canteen_label.clone();
    window.stendal_item.borrow_mut().connect_activate(move |item| {
        stendal_stack_handle.borrow()
            .set_visible_child_name("stendal");
        stendal_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let wernigerode_stack_handle = window.canteen_stack.clone();
    let wernigerode_label_handle = window.canteen_label.clone();
    window.wernigerode_item.borrow_mut().connect_activate(move |item| {
        wernigerode_stack_handle.borrow()
            .set_visible_child_name("wernigerode");
        wernigerode_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let dom_cafete_stack_handle = window.canteen_stack.clone();
    let dom_cafete_label_handle = window.canteen_label.clone();
    window.dom_cafete_item.borrow_mut().connect_activate(move |item| {
        dom_cafete_stack_handle.borrow()
            .set_visible_child_name("dom-cafete");
        dom_cafete_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let mut canteens = vec![
        (CanteenDescription::UniCampusLowerHall, window.lower_hall_days_box.clone()),
        (CanteenDescription::UniCampusUpperHall, window.upper_hall_days_box.clone()),
        (CanteenDescription::Kellercafe, window.kellercafe_days_box.clone()),
        (CanteenDescription::Herrenkrug, window.herrenkrug_days_box.clone()),
        (CanteenDescription::Stendal, window.stendal_days_box.clone()),
        (CanteenDescription::Wernigerode, window.wernigerode_days_box.clone()),
        (CanteenDescription::DomCafeteHalberstadt, window.dom_cafete_days_box.clone()),
    ];


    for (desc, days_box) in canteens.drain(..) {
        let (tx, rx) = channel();

        rt.spawn(async move {
            tx.send(Canteen::new(desc).await).unwrap();
        });

        let c = glib::MainContext::default();
        c.spawn_local(async move {
            if let Ok(mut canteen) = rx.await.unwrap() {
                for day in canteen.days.drain(..) {
                    let day_comp = DayComponent::new(&day).await;
                    days_box.borrow_mut().pack_start(&day_comp.frame, false, true, 0);
                    glib_yield!();
                }
            } else {
                // TODO: add error handling
            }
        });
    }

    window.window.borrow_mut().set_application(Some(app));
    window.window.borrow_mut().show_all();

    Ok(())
}

pub struct Application {
    pub g_app: gtk::Application,
    pub runtime: Runtime,
}

impl Application {
    pub fn new() -> Result<Application, &'static str> {
        gtk::init().map_err(|_| "Failed to initialize GTK!")?;

        let css_provider = CssProvider::new();
        css_provider
            .load_from_data(std::include_str!("../data/gnome-ovgu-canteen.css").as_bytes())
            .unwrap();

        let screen = Screen::get_default().ok_or(
            "Cannot find default screen!",
        )?;
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
            gtk::Application::new(Some("org.gnome.ovgu-canteen"), Default::default())
                .map_err(|_| "Failed to create application!")?;

        let build_rt = runtime.handle().clone();
        g_app.connect_activate(move |app| {
            match build(&build_rt, app) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("error: {}", err);
                    app.quit();
                },
            }
        });

        Ok(Application {
            g_app,
            runtime,
        })
    }

    pub fn run(self, args: Vec<String>) -> i32 {
        self.g_app.run(&args)
    }
}
