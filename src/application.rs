use ::std;
use ::gtk;
use ::gdk;
use ::gio;

use std::sync::Arc;
use widgets::{MainWidget, AppMenuWidget};
use gtk::prelude::*;

pub struct Widgets
{
    pub window: gtk::ApplicationWindow,
    pub main_widget: MainWidget,
    pub app_menu_widget: AppMenuWidget,
}

pub struct Application
{
    pub g_app: gtk::Application,
    pub builder: gtk::Builder,
    pub widgets: Widgets,
}

impl Application
{
    pub fn new() -> Result<Application, &'static str>
    {
        gtk::init().map_err(|_| "Failed to initialize GTK!")?;

        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_path("data/gnome-ovgu-canteen.css").map_err(|_| "Failed to load stylesheets!")?;

        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().ok_or("Cannot find default screen!")?,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER
        );

        let builder = gtk::Builder::new_from_file("data/gnome-ovgu-canteen.glade");

        Ok(Application
        {
            g_app: gtk::Application::new(Some("org.gnome.ovgu-canteen"), gio::APPLICATION_FLAGS_NONE)
                .map_err(|_| "Failed to create application!")?,
            widgets: Widgets
            {
                window: builder.get_object("window")
                    .ok_or("Cannot find window!")?,
                main_widget: MainWidget
                {
                    horizontal_navigation_stack: builder.get_object("horizontal-navigation-stack")
                        .ok_or("Cannot find horizontal navigation stack!")?,
                    back_button: builder.get_object("back_button")
                        .ok_or("Cannot find back button!")?,
                    canteen_list_box: builder.get_object("canteen-list-box")
                        .ok_or("Cannot find canteen list box!")?,
                },
                app_menu_widget: AppMenuWidget{},
            },
            builder: builder,
        })
    }

    pub fn startup(&self)
    {
        self.g_app.add_window(&self.widgets.window);
        self.widgets.app_menu_widget.startup(&self);
    }

    pub fn activate(&self)
    {
        self.widgets.window.show_all();
    }

    pub fn run(self)
    {
        let args1: Vec<String> = std::env::args().collect();
        let args2: Vec<&str> = args1.iter().map(AsRef::as_ref).collect();
        let argv: &[&str] = &args2;
        let app = Arc::new(self);

        {
            let binding = app.clone();
            app.g_app.connect_activate(move |_| {
                binding.activate();
            });
        }

        {
            let binding = app.clone();
            app.g_app.connect_startup(move |_| {
                binding.startup();
            });
        }

        app.widgets.main_widget.connect_signals(&app);
        std::process::exit(app.g_app.run(argv.len() as i32, argv));
    }
}
