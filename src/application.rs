use gtk::{Builder, CssProvider, Button, Stack, Label, AboutDialog};
use gtk::prelude::*;
use gio::prelude::*;
use gdk::{Screen};
use ovgu_canteen::{Canteen, CanteenDescription};
use tokio::runtime::Runtime;

use std::rc::Rc;
use std::cell::RefCell;

use crate::components::{GLADE, WindowComponent, DayComponent};

// TODO: persist canteen menus on disk for faster loading of app and update menus
//       when loaded

// TODO: add settings window with hamburger menu to access the settings

// TODO: add reload button for reloading canteen menus on network failure

// TODO: add dark theme to settings

// TODO: move about button to hamburger menu

// TODO: load each canteen independently and show result when done to enable
//       showing menus while others are still loading

// TODO: set default canteen in settings

// TODO: handle error when canteen menu failed to load by displaying error message

// TODO: show loading indicator while a canteen menu is fetched:
//        - when no data for the canteen is available show loading message in
//          canteens stack page
//        - animate the reload button

// TODO: set offset of canteen popup-menu so that the current item is on the
//       mouse position

// TODO: write readme

// TODO: create flatpak package

// TODO: try porting to windows metro app

// TODO: try porting to macos app

async fn build(app: &gtk::Application) -> Result<(), &'static str> {
    let builder = Builder::new_from_string(GLADE);

    let canteen_stack: Rc<RefCell<Stack>> = Rc::new(RefCell::new(
        builder.get_object("canteen-stack").unwrap()
    ));
    let canteen_label: Rc<RefCell<Label>> = Rc::new(RefCell::new(
        builder.get_object("canteen-label").unwrap()
    ));
    let window = WindowComponent {
        window: builder.get_object("window").unwrap(),
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
    };
    let about_dialog: AboutDialog = builder.get_object("about").unwrap();
    let about_button: Button = builder.get_object("about-btn").unwrap();

    about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
    about_button.connect_clicked(move |_btn| {
        about_dialog.run();
        about_dialog.hide();
    });

    let lower_hall_stack_handle = canteen_stack.clone();
    let lower_hall_label_handle = canteen_label.clone();
    window.lower_hall_item.connect_activate(move |item| {
        lower_hall_stack_handle.borrow()
            .set_visible_child_name("ovgu-lower-hall");
        lower_hall_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let upper_hall_stack_handle = canteen_stack.clone();
    let upper_hall_label_handle = canteen_label.clone();
    window.upper_hall_item.connect_activate(move |item| {
        upper_hall_stack_handle.borrow()
            .set_visible_child_name("ovgu-upper-hall");
        upper_hall_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let kellercafe_stack_handle = canteen_stack.clone();
    let kellercafe_label_handle = canteen_label.clone();
    window.kellercafe_item.connect_activate(move |item| {
        kellercafe_stack_handle.borrow()
            .set_visible_child_name("kellercafe");
        kellercafe_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let herrenkrug_stack_handle = canteen_stack.clone();
    let herrenkrug_label_handle = canteen_label.clone();
    window.herrenkrug_item.connect_activate(move |item| {
        herrenkrug_stack_handle.borrow()
            .set_visible_child_name("herrenkrug");
        herrenkrug_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let stendal_stack_handle = canteen_stack.clone();
    let stendal_label_handle = canteen_label.clone();
    window.stendal_item.connect_activate(move |item| {
        stendal_stack_handle.borrow()
            .set_visible_child_name("stendal");
        stendal_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let wernigerode_stack_handle = canteen_stack.clone();
    let wernigerode_label_handle = canteen_label.clone();
    window.wernigerode_item.connect_activate(move |item| {
        wernigerode_stack_handle.borrow()
            .set_visible_child_name("wernigerode");
        wernigerode_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let dom_cafete_stack_handle = canteen_stack.clone();
    let dom_cafete_label_handle = canteen_label.clone();
    window.dom_cafete_item.connect_activate(move |item| {
        dom_cafete_stack_handle.borrow()
            .set_visible_child_name("dom-cafete");
        dom_cafete_label_handle.borrow().set_text(&item.get_label().unwrap());
    });

    let (
        uni_campus_lower_hall,
        uni_campus_upper_hall,
        kellercafe,
        herrenkrug,
        stendal,
        wernigerode,
        dom_cafete,
    ) = tokio::try_join!(
        Canteen::new(CanteenDescription::UniCampusLowerHall),
        Canteen::new(CanteenDescription::UniCampusUpperHall),
        Canteen::new(CanteenDescription::Kellercafe),
        Canteen::new(CanteenDescription::Herrenkrug),
        Canteen::new(CanteenDescription::Stendal),
        Canteen::new(CanteenDescription::Wernigerode),
        Canteen::new(CanteenDescription::DomCafeteHalberstadt),
    ).map_err(|_| "Failed to fetch canteen menus!")?;

    for day in uni_campus_lower_hall.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.lower_hall_days_box.pack_start(&day_comp.frame, false, true, 0);
    }

    for day in uni_campus_upper_hall.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.upper_hall_days_box.pack_start(&day_comp.frame, false, true, 0);
    }

    for day in kellercafe.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.kellercafe_days_box.pack_start(&day_comp.frame, false, true, 0);
    }

    for day in herrenkrug.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.herrenkrug_days_box.pack_start(&day_comp.frame, false, true, 0);
    }

    for day in stendal.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.stendal_days_box.pack_start(&day_comp.frame, false, true, 0);
    }

    for day in wernigerode.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.wernigerode_days_box.pack_start(&day_comp.frame, false, true, 0);
    }

    for day in dom_cafete.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.dom_cafete_days_box.pack_start(&day_comp.frame, false, true, 0);
    }

    window.window.set_application(Some(app));
    window.window.show_all();

    Ok(())
}

pub struct Application {
    pub g_app: gtk::Application,
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

        let g_app =
            gtk::Application::new(Some("org.gnome.ovgu-canteen"), Default::default())
                .map_err(|_| "Failed to create application!")?;

        g_app.connect_activate(|app| {
            let mut rt = Runtime::new().unwrap();

            match rt.block_on(build(app)) {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("error: {}", err);
                    app.quit();
                },
            }
        });

        Ok(Application {
            g_app,
        })
    }

    pub fn run(self, args: Vec<String>) -> i32 {
        self.g_app.run(&args)
    }
}
