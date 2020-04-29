use anyhow::Result;
use glib::SignalHandlerId;
use gio::prelude::*;
use gio::Settings;
use gtk::prelude::*;
use gtk::{Builder, Window, Switch, ComboBox, SpinButton, ListStore};
use ovgu_canteen::CanteenDescription;

use crate::components::{get, GLADE};
use crate::util::enclose;
use crate::canteen;

pub fn open<'a, I: IntoIterator<Item = &'a CanteenDescription>>(parent_window: &Window, settings: &Settings, canteens: I) -> Result<()> {
    let builder = Builder::new_from_string(GLADE);
    let preferences: Window = get!(&builder, "preferences")?;
    let dark_theme_switch: Switch = get!(&builder, "dark-theme-switch")?;
    let default_canteen_combo_box: ComboBox = get!(&builder, "default-canteen-combo-box")?;
    let canteen_list_store: ListStore = get!(&builder, "canteen-liststore")?;
    let menu_history_length_spin_button: SpinButton = get!(&builder, "menu-history-length-spin-button")?;

    for (idx, canteen) in canteens.into_iter().enumerate() {
        canteen_list_store.insert_with_values(
            Some(idx as u32),
            &[0, 1],
            &[&canteen::translate(&canteen), &serde_plain::to_string(&canteen).unwrap()],
        );
    }

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

    preferences.show_all();

    Ok(())
}
