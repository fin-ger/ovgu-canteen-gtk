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
    log::debug!("updating cache-size label in preferences");

    let file = xdg::BaseDirectories::with_prefix("ovgu-canteen-gtk").ok()
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

    log::debug!("new cache-size shown in preferences is {}", humansize);

    cache_size_label.set_text(&humansize);
}

pub fn open<'a, I: IntoIterator<Item = &'a CanteenDescription>>(rt: &Handle, window: &WindowComponent, canteens: I) -> Result<()> {
    log::debug!("opening up preferences");

    let builder = Builder::new_from_string(GLADE);
    let preferences: Window = get!(&builder, "preferences")?;
    let dark_theme_switch: Switch = get!(&builder, "dark-theme-switch")?;
    let default_canteen_combo_box: ComboBox = get!(&builder, "default-canteen-combo-box")?;
    let canteen_list_store: ListStore = get!(&builder, "canteen-liststore")?;
    let menu_history_length_spin_button: SpinButton = get!(&builder, "menu-history-length-spin-button")?;
    let clear_cache_button: Button = get!(&builder, "clear-cache-button")?;
    let cache_size_label: Label = get!(&builder, "cache-size-label")?;

    log::debug!("inserting available canteens into preferences combo-box");
    for (idx, canteen) in canteens.into_iter().enumerate() {
        canteen_list_store.insert_with_values(
            Some(idx as u32),
            &[0, 1],
            &[&canteen::translate(&canteen), &serde_plain::to_string(&canteen).unwrap()],
        );
    }

    // watch for changes on canteen cache and notify UI thread over tx/rx channel of the changes
    let (mut tx, mut rx) = channel(32); // 32 filesystem change events can be buffered
    // notification that preferences window has been closed
    let quit_send = Arc::new(Notify::new());
    rt.spawn(enclose! { (quit_send) async move {
        let (std_tx, std_rx) = std::sync::mpsc::channel();
        // install filesystem watcher
        let mut watcher = watcher(std_tx, std::time::Duration::from_millis(100)).unwrap();
        for xdg in xdg::BaseDirectories::with_prefix("ovgu-canteen-gtk") {
            watcher.watch(xdg.get_cache_home(), RecursiveMode::NonRecursive).ok();

            loop {
                // check if new event are available from watcher
                let val = std_rx.try_recv();
                match val {
                    Ok(event) => {
                        log::debug!("a new cache change event was received: {:?}", event);
                        if let Err(e) = tx.send(event).await {
                            log::debug!("an error occurred notifying the ui thread of cache changes: {:#}", e);
                            // quit if error occurred
                            break;
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // use polling here as notify-rs can only work with std-channels
                        tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
                    },
                    // quit if error occurred
                    Err(e) => {
                        log::debug!("an error occurred while watching for cache changes: {:#}", e);
                        break;
                    },
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
            // wait for a new cache change event or a quit notification, whatever comes first
            match future::select(rx.recv().boxed(), quit_recv.notified().boxed()).await {
                Either::Left((Some(_event), _quit_future)) => {
                    update_cache_size_label(&cache_size_label);
                },
                _ => {
                    // quit if notified or any error occurred
                    log::debug!("an error occurred waiting for cache changes");
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

    log::debug!("loading current settings into preferences");
    dark_theme_switch.set_state(settings.get_boolean("dark-theme-variant"));
    if let Some(canteen) = settings.get_string("default-canteen") {
        default_canteen_combo_box.set_active_id(Some(&canteen));
    }
    menu_history_length_spin_button.set_value(settings.get_uint64("menu-history-length") as f64);

    log::debug!("connecting settings-changed handlers");
    let signal_handler = settings.connect_changed(enclose! {
        (
            dark_theme_switch,
            default_canteen_combo_box,
            menu_history_length_spin_button,
        ) move |settings, key| {
            match key {
                "dark-theme-variant" => {
                    log::debug!("dark-theme-variant changed to {}", settings.get_boolean(key));
                    dark_theme_switch.set_state(settings.get_boolean(key));
                },
                "default-canteen" => {
                    if let Some(canteen) = settings.get_string(key) {
                        log::debug!("default-canteen changed to {}", canteen);
                        default_canteen_combo_box.set_active_id(Some(&canteen));
                    }
                },
                "menu-history-length" => {
                    log::debug!("menu-history-length changed to {}", settings.get_uint64(key));
                    menu_history_length_spin_button.set_value(settings.get_uint64(key) as f64);
                },
                _ => {},
            }
        }
    });

    log::debug!("connecting UI signals for preferences");

    // when preferences get closed, cleanup
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
        // start removing the cache in a tokio future
        let removed = Arc::new(Notify::new());
        rt.spawn(enclose! { (removed) async move {
            log::debug!("try removing cache");
            // loop will only run once, used to abort early with break as ? is not available in scopes
            for xdg in xdg::BaseDirectories::with_prefix("ovgu-canteen-gtk") {
                log::debug!("found cache directory");
                let history_path = match xdg.find_cache_file("history.json") {
                    Some(path) => path,
                    // if no cache is available, just skip
                    None => {
                        log::info!("no cache available in cache directory");
                        break;
                    },
                };
                log::debug!("found cache history file in {:?}", history_path);

                // this cannot fail, as xdg.find_cache_file makes sure the file exists
                if let Err(err) = tokio::fs::remove_file(history_path).await {
                    log::error!("failed removing cache: {:#}", err);
                    break;
                }

                log::debug!("notifying preferences window that cache got removed");
                removed.notify();
            }
        }});

        let c = glib::MainContext::default();
        c.spawn_local(enclose! { (window, rt, btn) async move {
            log::debug!("waiting for cache for removed");
            btn.set_sensitive(false);

            removed.notified().await;
            log::debug!("cache got removed, reloading canteens...");

            let loaded = Arc::new(Notify::new());
            window.load(&rt, Some(loaded.clone()));
            loaded.notified().await;
            log::debug!("canteens got reloaded");

            btn.set_sensitive(true);
            log::debug!("finish removing cache");
        }});
    }});

    log::debug!("showing preferences");

    preferences.show_all();

    log::debug!("finish opening up preferences");

    Ok(())
}
