extern crate gtk;
extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gio_sys;
extern crate glib_sys;
extern crate libc;

use std::sync::Arc;
use glib::translate::ToGlibPtr;
use gtk::prelude::*;

type ActionCallback = unsafe extern "C" fn(*mut gio_sys::GSimpleAction, *mut glib_sys::GVariant, *mut libc::c_void);

fn main()
{
    let app = Application::new();
    app.run();
}

struct ActionEntry<'a>
{
    name: &'a str,
    parameter_type: Option<&'a str>,
    state: Option<&'a str>,
    activate: Option<ActionCallback>,
    change_state: Option<ActionCallback>,
}

impl<'a> ActionEntry<'a>
{
    pub fn new<P: Into<Option<&'a str>>, S: Into<Option<&'a str>>>
        (name: &'a str,
         activate: Option<ActionCallback>, parameter_type: P,
         change_state: Option<ActionCallback>, state: S) -> ActionEntry<'a>
    {
        ActionEntry
        {
            name: name,
            activate: activate.into(),
            parameter_type: parameter_type.into(),
            state: state.into(),
            change_state: change_state.into(),
        }
    }
}

fn g_action_map_add_action_entries<'a, I, T>(gapp: &gtk::Application, entries: I, user_data: &T)
    where I: Iterator<Item=&'a ActionEntry<'a>>
{
    // cache the stashes from `to_glib_none` to avoid them going out of scope
    let mut names = vec![];
    let mut params = vec![];
    let mut states = vec![];

    let mut vec = entries.map(|entry| {
        names.push(entry.name.to_glib_none());
        params.push(entry.parameter_type.to_glib_none());
        states.push(entry.state.to_glib_none());

        gio_sys::GActionEntry
        {
            name: names.last().unwrap().0,
            activate: entry.activate,
            parameter_type: params.last().unwrap().0,
            state: states.last().unwrap().0,
            change_state: entry.change_state,
            padding: [0, 0, 0],
        }
    }).collect::<Vec<gio_sys::GActionEntry>>();
    let slice = vec.as_mut_slice();

    unsafe
    {
        gio_sys::g_action_map_add_action_entries(
            gapp.to_glib_none().0,
            slice.as_mut_ptr(),
            slice.len() as i32,
            user_data as *const _ as *mut libc::c_void
        );
    };
}

unsafe extern fn action_entry_activate(
    action: *mut gio_sys::GSimpleAction,
    _: *mut glib_sys::GVariant,
    user_data: glib_sys::gpointer
)
{
    let cname = std::ffi::CStr::from_ptr(
        gio_sys::g_action_get_name(action as *mut gio_sys::GAction) as *const _
    );
    let app = (user_data as *const Application).as_ref().unwrap();

    match cname.to_str().unwrap()
    {
        "prefs" => app.app_menu_prefs(),
        "about" => app.app_menu_about(),
        "quit" => app.app_menu_quit(),
        _ => println!("No handler registered for action '{}'!", cname.to_str().unwrap()),
    };
}

struct Application
{
    application: gtk::Application,
    builder: gtk::Builder,
    window: gtk::ApplicationWindow,
    horiz_nav: gtk::Stack,
    back_button: gtk::Button,
}

impl Application
{
    pub fn new() -> Application
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

        Application{
            application: gtk::Application::new(Some("org.gnome.ovgu-canteen"), gio::APPLICATION_FLAGS_NONE).unwrap(),
            window: builder.get_object("window").unwrap(),
            horiz_nav: builder.get_object("horizontal-navigation-stack").unwrap(),
            back_button: builder.get_object("back_button").unwrap(),
            builder: builder,
        }
    }

    pub fn startup(&self)
    {
        self.application.add_window(&self.window);

        let entries = [
            ActionEntry::new("prefs", Some(action_entry_activate), None, None, None),
            ActionEntry::new("about", Some(action_entry_activate), None, None, None),
            ActionEntry::new("quit", Some(action_entry_activate), None, None, None),
        ];

        g_action_map_add_action_entries(&self.application, entries.iter(), self);

        let app_menu = gio::Menu::new();

        let prefs_section = gio::Menu::new();
        prefs_section.append("_Preferences", "app.prefs");

        let general_section = gio::Menu::new();
        general_section.append("_About", "app.about");
        general_section.append("_Quit", "app.quit");

        app_menu.append_section(None, &prefs_section);
        app_menu.append_section(None, &general_section);

        self.application.set_app_menu(&app_menu);
    }

    pub fn app_menu_prefs(&self)
    {
        println!("Preferences");
    }

    pub fn app_menu_about(&self)
    {
        println!("About");
    }

    pub fn app_menu_quit(&self)
    {
        self.application.quit();
    }

    pub fn activate(&self)
    {
        self.window.show_all();
    }

    pub fn run(self)
    {
        let args1: Vec<String> = std::env::args().collect();
        let args2: Vec<&str> = args1.iter().map(AsRef::as_ref).collect();
        let argv: &[&str] = &args2;

        let canteen_list_box: gtk::ListBox = self.builder.get_object("canteen-list-box").unwrap();
        let app = Arc::new(self);

        {
            let my_app = app.clone();
            app.application.connect_activate(move |_| {
                my_app.activate()
            });
        }

        {
            let my_app = app.clone();
            app.application.connect_startup(move |_| {
                my_app.startup()
            });
        }

        {
            let my_app = app.clone();
            canteen_list_box.connect_row_activated(move |list_box, row| {
                my_app.canteen_activated(list_box, row)
            });
        }

        {
            let my_app = app.clone();
            app.back_button.connect_clicked(move |button| {
                my_app.back_clicked(button)
            });
        }

        std::process::exit(app.application.run(argv.len() as i32, argv));
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
