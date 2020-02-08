use gtk::{Builder, CssProvider};
use gtk::prelude::*;
use gio::prelude::*;
use gdk::{Screen};
use ovgu_canteen::{Canteen, CanteenDescription};
use tokio::runtime::Runtime;

use crate::components::{GLADE, WindowComponent, AboutComponent, DayComponent};

async fn build(app: &gtk::Application) -> Result<(), &'static str> {
    let builder = Builder::new_from_string(GLADE);

    let window = WindowComponent {
        window: builder.get_object("window").unwrap(),
        lower_hall_days_box: builder.get_object("lower-hall-days-box").unwrap(),
        upper_hall_days_box: builder.get_object("upper-hall-days-box").unwrap(),
    };
    // TODO: add about button and show about widget
    let about = AboutComponent {
        dialog: builder.get_object("about").unwrap(),
    };

    let lower_hall = Canteen::new(CanteenDescription::Downstairs).await
        .map_err(|_| "Failed to fetch lower hall menu!")?;
    let upper_hall = Canteen::new(CanteenDescription::Upstairs).await
        .map_err(|_| "Failed to fetch upper hall menu!")?;

    for day in lower_hall.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.lower_hall_days_box.pack_end(&day_comp.frame, false, true, 0);
    }

    for day in upper_hall.days.iter() {
        let day_comp = DayComponent::new(&day);
        window.upper_hall_days_box.pack_end(&day_comp.frame, false, true, 0);
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

            // TODO: load canteens in background and show spinner until data is available
            //       pass loaded canteens to build()
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
