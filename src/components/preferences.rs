use std::fs::File;

use anyhow::Result;
use glib::SignalHandlerId;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Builder, Button, Label, Window, Switch, ComboBox, SpinButton, ListStore};
use humansize::{FileSize, file_size_opts};
use ovgu_canteen::CanteenDescription;
use gettextrs::gettext as t;
use tokio::runtime::Handle;

use crate::components::{get, WindowComponent, GLADE};
use crate::util::enclose;
use crate::canteen;

// TODO: make cache size label update when file changes

fn update_cache_size_label(cache_size_label: &Label) {
    let file = xdg::BaseDirectories::new().ok()
        .and_then(|xdg| xdg.find_cache_file("gnome-ovgu-canteen/history.json"))
        .map(|path| {
            // this cannot fail, as xdg.find_cache_file makes sure the file exists
            File::open(path).unwrap()
        });
    let size = match file {
        Some(file) => file.metadata().ok().map(|meta| meta.len()),
        None => Some(0),
    };
    let humansize = size
        .and_then(|size| {
            size.file_size(file_size_opts::BINARY).ok()
        })
        .unwrap_or(t("Unknown"));

    cache_size_label.set_text(&humansize);
}

pub fn open<'a, I: IntoIterator<Item = &'a CanteenDescription>>(rt: &Handle, window: &WindowComponent, canteens: I) -> Result<()> {
    let builder = Builder::new_from_string(GLADE);
    let preferences: Window = get!(&builder, "preferences")?;
    let dark_theme_switch: Switch = get!(&builder, "dark-theme-switch")?;
    let default_canteen_combo_box: ComboBox = get!(&builder, "default-canteen-combo-box")?;
    let canteen_list_store: ListStore = get!(&builder, "canteen-liststore")?;
    let menu_history_length_spin_button: SpinButton = get!(&builder, "menu-history-length-spin-button")?;
    let clear_cache_button: Button = get!(&builder, "clear-cache-button")?;
    let cache_size_label: Label = get!(&builder, "cache-size-label")?;

    for (idx, canteen) in canteens.into_iter().enumerate() {
        canteen_list_store.insert_with_values(
            Some(idx as u32),
            &[0, 1],
            &[&canteen::translate(&canteen), &serde_plain::to_string(&canteen).unwrap()],
        );
    }

    update_cache_size_label(&cache_size_label);

    let parent_window = window.window();
    let settings = window.settings();

    if let Some(application) = parent_window.get_application() {
        preferences.set_application(Some(&application));
    }
    preferences.set_transient_for(Some(parent_window));
    preferences.set_attached_to(Some(parent_window));

    dark_theme_switch.set_state(settings.get_boolean("dark-theme-variant"));
    if let Some(canteen) = settings.get_string("default-canteen") {
        default_canteen_combo_box.set_active_id(Some(&canteen));
    }
    menu_history_length_spin_button.set_value(settings.get_uint64("menu-history-length") as f64);

    let signal_handler = settings.connect_changed(enclose! {
        (
            dark_theme_switch,
            default_canteen_combo_box,
            menu_history_length_spin_button,
        ) move |settings, key| {
            match key {
                "dark-theme-variant" => {
                    dark_theme_switch.set_state(settings.get_boolean(key));
                },
                "default-canteen" => {
                    if let Some(canteen) = settings.get_string(key) {
                        default_canteen_combo_box.set_active_id(Some(&canteen));
                    }
                },
                "menu-history-length" => {
                    menu_history_length_spin_button.set_value(settings.get_uint64(key) as f64);
                },
                _ => {},
            }
        }
    });

    preferences.connect_destroy(enclose! { (settings) move |_window| {
        use glib::translate::{FromGlib, ToGlib}; // clone or copy would be boring...
        settings.disconnect(SignalHandlerId::from_glib(signal_handler.to_glib()));
    }});

    dark_theme_switch.connect_state_set(enclose! { (settings) move |_switch, state| {
        settings.set_boolean("dark-theme-variant", state).unwrap();
        Inhibit(false)
    }});

    default_canteen_combo_box.connect_changed(enclose! { (settings) move |combo_box| {
        if let Some(canteen) = combo_box.get_active_id() {
            settings.set_string("default-canteen", &canteen).unwrap();
        }
    }});

    menu_history_length_spin_button.connect_changed(enclose! { (settings) move |spin_button| {
        settings.set_uint64("menu-history-length", spin_button.get_value() as u64).unwrap();
    }});

    clear_cache_button.connect_clicked(enclose! { (window, rt, cache_size_label) move |btn| {
        btn.set_sensitive(false);

        // TODO: add async file io to not block UI thread when deleting
        // loop will only run once, used to abort early with break as ? is not available in scopes
        for xdg in xdg::BaseDirectories::new() {
            let history_path = match xdg.find_cache_file("gnome-ovgu-canteen/history.json") {
                Some(path) => path,
                // if no cache is available, just skip
                None => break,
            };

            // this cannot fail, as xdg.find_cache_file makes sure the file exists
            if let Err(_err) = std::fs::remove_file(history_path) {
                // TODO: log error
                break;
            }

            update_cache_size_label(&cache_size_label);
            window.load(&rt);
        }
        btn.set_sensitive(true);
    }});

    preferences.show_all();

    Ok(())
}
