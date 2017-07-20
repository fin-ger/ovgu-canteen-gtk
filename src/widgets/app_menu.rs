use g_action_map::{ActionEntry, action_entry_activate, g_action_map_add_action_entries};
use gio::{Menu};
use gtk::prelude::*;
use application::Application;

pub struct AppMenuWidget;

impl AppMenuWidget
{
    pub fn startup(&self, app: &Application)
    {
        let entries = [
            ActionEntry::new("prefs", Some(action_entry_activate), None, None, None),
            ActionEntry::new("about", Some(action_entry_activate), None, None, None),
            ActionEntry::new("quit", Some(action_entry_activate), None, None, None),
        ];

        g_action_map_add_action_entries(&app.g_app, entries.iter(), app);

        let app_menu = Menu::new();

        let prefs_section = Menu::new();
        prefs_section.append("_Preferences", "app.prefs");

        let general_section = Menu::new();
        general_section.append("_About", "app.about");
        general_section.append("_Quit", "app.quit");

        app_menu.append_section(None, &prefs_section);
        app_menu.append_section(None, &general_section);

        app.g_app.set_app_menu(&app_menu);
    }

    pub fn prefs(&self, _app: &Application)
    {
        println!("Preferences");
    }

    pub fn about(&self, _app: &Application)
    {
        println!("About");
    }

    pub fn quit(&self, app: &Application)
    {
        app.g_app.quit();
    }
}
