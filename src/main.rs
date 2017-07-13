extern crate gtk;
extern crate gdk;
extern crate gio;

use std::sync::Arc;
use gtk::prelude::*;

fn main()
{
    let app = Application::new();
    app.run();
}

struct Application
{
    builder: gtk::Builder,
    window: gtk::Window,
    horiz_nav: gtk::Stack,
    back_button: gtk::Button,
}

impl Application
{
    pub fn new() -> Arc<Application>
    {
        if gtk::init().is_err()
        {
            println!("Failed to initialize GTK.");
            std::process::exit(1);
        }

        let builder = gtk::Builder::new_from_file("data/gnome-ovgu-canteen.glade");

        let css_provider = gtk::CssProvider::new();

        if css_provider.load_from_path("data/gnome-ovgu-canteen.css").is_err()
        {
            println!("Failed to load stylesheets.");
            std::process::exit(2);
        }

        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER
        );

        let app = Arc::new(Application{
            window: builder.get_object("window").unwrap(),
            horiz_nav: builder.get_object("horizontal-navigation-stack").unwrap(),
            back_button: builder.get_object("back_button").unwrap(),
            builder: builder,
        });

        app.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        {
            let canteen_list_box: gtk::ListBox = app.builder.get_object("canteen-list-box").unwrap();
            let app = app.clone();
            canteen_list_box.connect_row_activated(move |list_box, row| {
                Self::canteen_activated(&app, list_box, row)
            });
        }

        {
            let my_app = app.clone();
            app.back_button.connect_clicked(move |button| {
                Self::back_clicked(&my_app, button)
            });
        }

        app
    }

    pub fn run(&self)
    {
        self.window.show_all();
        /*let app_menu = gio::Menu::new();
        app_menu.append("Preferences", "app.prefs");
        app_menu.append("About", "app.about");
        app_menu.append("Quit", "app.quit");
        self.window.get_application().unwrap().set_app_menu(&app_menu);*/
        gtk::main();
    }

    pub fn canteen_activated(&self, _list_box: &gtk::ListBox, _row: &gtk::ListBoxRow)
    {
        self.horiz_nav.set_visible_child_name("menu");
        self.back_button.set_sensitive(true);

    }

    pub fn back_clicked(&self, _button: &gtk::Button)
    {
        self.horiz_nav.set_visible_child_name("canteens");
        self.back_button.set_sensitive(false);
    }
}
