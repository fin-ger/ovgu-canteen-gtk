use std::sync::Arc;
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
use tokio::sync::Notify;
use tokio::sync::mpsc::channel;
use notify::{RecursiveMode, watcher, Watcher};
use futures::future::{self, Either, FutureExt};

use crate::components::{get, WindowComponent, GLADE};
use crate::util::enclose;
use crate::canteen;

fn update_cache_size_label(cache_size_label: &Label) {
    let file = xdg::BaseDirectories::with_prefix("gnome-ovgu-canteen").ok()
        .and_then(|xdg| xdg.find_cache_file("history.json"))
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

    let (mut tx, mut rx) = channel(32);
    let quit_send = Arc::new(Notify::new());
    rt.spawn(enclose! { (quit_send) async move {
        let (std_tx, std_rx) = std::sync::mpsc::channel();
        let mut watcher = watcher(std_tx, std::time::Duration::from_millis(100)).unwrap();
        for xdg in xdg::BaseDirectories::with_prefix("gnome-ovgu-canteen") {
            watcher.watch(xdg.get_cache_home(), RecursiveMode::NonRecursive).ok();

            loop {
                let val = std_rx.try_recv();
                match val {
                    Ok(event) => {
                        if let Err(_) = tx.send(event).await {
                            // quit if error occurred
                            break;
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // use polling here as notify-rs can only work with std-channels
                        tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
                    },
                    // quit if error occurred
                    _ => break,
                }

                if let Some(_) = quit_send.notified().now_or_never() {
                    // quit if notified
                    break;
                }
            }
        }
    }});

    let c = glib::MainContext::default();
    let quit_recv = Arc::new(Notify::new());
    c.spawn_local(enclose! { (quit_recv, cache_size_label) async move {
        loop {
            match future::select(rx.recv().boxed(), quit_recv.notified().boxed()).await {
                Either::Left((Some(_event), _quit_future)) => {
                    update_cache_size_label(&cache_size_label);
                },
                _ => {
                    // quit if notified or any error occurred
                    break;
                },
            }
        }
    }});

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
        // quit the file watcher and label updater futures
        quit_recv.notify();
        quit_send.notify();

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

    clear_cache_button.connect_clicked(enclose! { (window, rt) move |btn| {
        let removed = Arc::new(Notify::new());
        rt.spawn(enclose! { (removed) async move {
            // loop will only run once, used to abort early with break as ? is not available in scopes
            for xdg in xdg::BaseDirectories::with_prefix("gnome-ovgu-canteen") {
                let history_path = match xdg.find_cache_file("history.json") {
                    Some(path) => path,
                    // if no cache is available, just skip
                    None => break,
                };

                // this cannot fail, as xdg.find_cache_file makes sure the file exists
                if let Err(_err) = tokio::fs::remove_file(history_path).await {
                    // TODO: log error
                    break;
                }

                removed.notify();
            }
        }});

        let c = glib::MainContext::default();
        c.spawn_local(enclose! { (window, rt, btn) async move {
            btn.set_sensitive(false);

            removed.notified().await;
            let loaded = Arc::new(Notify::new());
            window.load(&rt, Some(loaded.clone()));
            loaded.notified().await;

            btn.set_sensitive(true);
        }});
    }});

    preferences.show_all();

    Ok(())
}
