use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;
use std::sync::Arc;

use lazy_static::lazy_static;
use anyhow::{bail, Context, Result};
use cargo_author::Author;
use gio::prelude::*;
use gio::{Settings, SimpleAction};
use gtk::prelude::*;
use gtk::{
    AboutDialog, Box, Builder, Button, ButtonRole, Label, MenuButton, ModelButtonBuilder, Stack,
    Window
};
use ovgu_canteen::{Canteen, CanteenDescription};
use send_wrapper::SendWrapper;
use tokio::runtime::Handle;
use tokio::sync::mpsc::channel;
use tokio::sync::Notify;
use futures::future;
use chrono::{Local, Duration};
use gettextrs::gettext as t;

use crate::components::{get, preferences, CanteenComponent, GLADE};
use crate::util::enclose;
use crate::canteen;

lazy_static! {
    // all available canteens
    static ref CANTEENS: Vec<CanteenDescription> = vec![
        CanteenDescription::UniCampusLowerHall,
        CanteenDescription::UniCampusUpperHall,
        CanteenDescription::Kellercafe,
        CanteenDescription::Herrenkrug,
        CanteenDescription::Stendal,
        CanteenDescription::Wernigerode,
        CanteenDescription::DomCafeteHalberstadt,
    ];
}

#[derive(Clone)]
pub struct WindowComponent {
    window: Window,
    window_stack: Stack,
    window_error_label: Label,
    canteens_stack: Stack,
    canteens_menu: Box,
    canteen_menu_button: MenuButton,
    canteen_label: Label,
    reload_button: Button,
    canteen_components: Rc<RefCell<HashMap<CanteenDescription, CanteenComponent>>>,
    settings: Settings,
}

impl WindowComponent {
    pub fn new(rt: &Handle, app: &gtk::Application) -> Result<()> {
        log::debug!("creating new WindowComponent");

        log::debug!("fetching settings for application");
        let settings = Settings::new("io.github.fin_ger.OvGUCanteen");
        settings.connect_changed(|settings, key| {
            match key {
                "dark-theme-variant" => {
                    if let Some(gtk_settings) = gtk::Settings::get_default() {
                        log::debug!("setting dark theme to {}", settings.get_boolean(key));
                        gtk::SettingsExt::set_property_gtk_application_prefer_dark_theme(
                            &gtk_settings,
                            settings.get_boolean(key),
                        );
                    }
                },
                _ => {},
            }
        });
        if let Some(gtk_settings) = gtk::Settings::get_default() {
            log::debug!("setting dark theme to {}", settings.get_boolean("dark-theme-variant"));
            gtk::SettingsExt::set_property_gtk_application_prefer_dark_theme(
                &gtk_settings,
                settings.get_boolean("dark-theme-variant"),
            );
        }

        log::debug!("loading UI for WindowComponent");

        let builder = Builder::new_from_string(GLADE);

        let window: Window = get!(&builder, "window")?;
        let window_stack: Stack = get!(&builder, "window-stack")?;
        let window_error_label: Label = get!(&builder, "window-error-label")?;
        let canteens_stack: Stack = get!(&builder, "canteens-stack")?;
        let canteens_menu: Box = get!(&builder, "canteens-menu")?;
        let canteen_label: Label = get!(&builder, "canteen-label")?;
        let canteen_menu_button: MenuButton = get!(&builder, "canteen-menu-button")?;
        let about_dialog: AboutDialog = get!(&builder, "about")?;
        let preferences_button: Button = get!(&builder, "preferences-btn")?;
        let about_button: Button = get!(&builder, "about-btn")?;
        let options_button: MenuButton = get!(&builder, "options-button")?;
        let reload_button: Button = get!(&builder, "reload-button")?;

        window.set_application(Some(app));
        window.set_icon_name(Some("io.github.fin_ger.OvGUCanteen"));
        about_dialog.set_logo_icon_name(Some("io.github.fin_ger.OvGUCanteen.About"));

        let authors = env!("CARGO_PKG_AUTHORS")
            .split(':')
            .map(|author| Author::new(author))
            .collect::<Vec<_>>();

        about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
        about_dialog.set_website(Some(env!("CARGO_PKG_REPOSITORY")));
        about_dialog.set_website_label(Some("Source Code"));
        about_dialog.set_comments(Some(env!("CARGO_PKG_DESCRIPTION")));
        about_dialog.set_authors(
            &authors
                .iter()
                .map(|author| {
                    if let Some(name) = &author.name {
                        Ok(name.as_str())
                    } else if let Some(email) = &author.email {
                        Ok(email.as_str())
                    } else if let Some(url) = &author.url {
                        Ok(url.as_str())
                    } else {
                        bail!("Failed to get author name");
                    }
                })
                .collect::<Result<Vec<_>>>()?,
        );

        about_button.connect_clicked(enclose! { (options_button) move |_btn| {
            if let Some(popover) = options_button.get_popover() {
                popover.popdown();
            }

            about_dialog.run();
            about_dialog.hide();
        }});

        let canteen_selected_action = SimpleAction::new(
            // action name
            "canteen-selected",
            // single parameter which is a string containing the canteen-name
            Some(glib::VariantTy::new("s").unwrap()),
        );
        let canteens_stack_handle = canteens_stack.clone();
        let canteen_label_handle = canteen_label.clone();
        canteen_selected_action.connect_activate(move |_action, maybe_canteen_variant| {
            let canteen_variant = match maybe_canteen_variant {
                Some(v) => v,
                None => return,
            };
            let canteen_name = match canteen_variant.get_str() {
                Some(s) => s,
                None => return,
            };

            log::debug!("switching visible canteen to {}", canteen_name);
            canteens_stack_handle.set_visible_child_name(canteen_name);
            canteen_label_handle.set_text(
                &canteen::translate(
                    &serde_plain::from_str::<CanteenDescription>(canteen_name).unwrap()
                )
            );
        });
        app.add_action(&canteen_selected_action);

        log::debug!("showing window");

        window.show_all();

        let comp = Self {
            window,
            window_stack,
            window_error_label,
            canteens_stack,
            canteens_menu,
            canteen_label,
            canteen_menu_button,
            reload_button,
            canteen_components: Rc::new(RefCell::new(HashMap::new())),
            settings,
        };

        preferences_button.connect_clicked(enclose! { (rt, comp, options_button) move |_btn| {
            if let Some(popover) = options_button.get_popover() {
                popover.popdown();
            }

            let _preferences = preferences::open(&rt, &comp, CANTEENS.iter());
        }});

        log::debug!("creating CanteenComponents");
        let mut canteen_components_borrow = comp.canteen_components.borrow_mut();
        for desc in CANTEENS.iter() {
            canteen_components_borrow.insert(
                desc.clone(),
                CanteenComponent::new(desc, &comp).context("Failed to create canteen!")?,
            );
        }
        drop(canteen_components_borrow);

        log::debug!("make default canteen visible");
        if let Some(default_canteen) = comp.settings.get_string("default-canteen") {
            log::debug!("switching visible canteen to {}", default_canteen);
            comp.canteens_stack.set_visible_child_name(&default_canteen);
            comp.canteen_label.set_text(
                &canteen::translate(
                    &serde_plain::from_str::<CanteenDescription>(&default_canteen).unwrap()
                )
            );
        }

        log::debug!("loading CanteenComponents");
        comp.load(rt, None);
        comp.reload_button
            .clone()
            .connect_clicked(enclose! { (rt) move |_btn| {
                log::debug!("reloading CanteenComponents");
                comp.load(&rt, None);
            }});

        log::debug!("finish creating WindowComponent");

        Ok(())
    }

    pub fn add_canteen(&self, canteen_stack: &Stack, canteen: String, canteen_name: String) -> Result<()> {
        log::debug!("adding canteen {} to WindowComponent", canteen_name);
        self.canteens_stack.add_named(canteen_stack, &canteen);

        let model_btn = ModelButtonBuilder::new()
            .visible(true)
            .text(&canteen_name)
            .can_focus(false)
            .action_name("app.canteen-selected")
            .action_target(&canteen.to_variant())
            .role(ButtonRole::Radio)
            .build();

        self.canteens_menu.pack_start(&model_btn, false, true, 0);

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    #[cfg(feature = "test-with-local-files")]
    async fn load_canteen(cached_canteen: Option<Canteen>, canteen_desc: CanteenDescription) -> Result<Canteen> {
        log::debug!("loading canteen {:?} from local file", canteen_desc);
        let file = File::open("data/canteens.json").context("'data/canteens.json' not found!")?;
        let mut canteens: Vec<Canteen> =
            serde_json::from_reader(&file).context("Could not parse 'data/cateens.json'")?;
        let canteen = canteens
            .drain(..)
            .find(|c| c.description == canteen_desc)
            .context("Canteen not found!")?;

        if let Some(mut cached_canteen) = cached_canteen {
            failure::ResultExt::compat(cached_canteen.merge(canteen))
                .context("Failed to update canteen")?;
            Ok(cached_canteen)
        } else {
            Ok(canteen)
        }
    }

    #[cfg(not(feature = "test-with-local-files"))]
    async fn load_canteen(cached_canteen: Option<Canteen>, canteen_desc: CanteenDescription) -> Result<Canteen> {
        log::debug!("loading canteen {:?}", canteen_desc);
        if let Some(mut canteen) = cached_canteen {
            failure::ResultExt::compat(canteen.update().await)
                .context("Failed to update canteen")?;
            Ok(canteen)
        } else {
            failure::ResultExt::compat(Canteen::new(canteen_desc.clone()).await)
                .context("Failed to fetch canteen")
        }
    }

    pub fn load(&self, rt: &Handle, loaded: Option<Arc<Notify>>) {
        log::debug!("loading canteens into WindowComponent");

        self.reload_button.set_sensitive(false);
        self.window_stack.set_visible_child_name("canteens-stack");

        let menu_history_length = self.settings.get_uint64("menu-history-length");
        let history_duration = Duration::days(menu_history_length as i64);
        let history_oldest = Local::now().date().naive_local() - history_duration;

        // canteens are downloaded in parallel here,
        // but in order for one canteen to show up in a batch
        // we are using an mpsc channel to put the parallel loaded canteens
        // in an order which is later sequentially inserted into the GUI.
        let (tx, mut rx) = channel(self.canteen_components.borrow().len());

        rt.spawn(async move {
            log::debug!("loading canteens from cache");
            let mut canteen_cache = HashMap::new();

            // loop will only run once, used to abort early with break as ? is not available in scopes
            for xdg in xdg::BaseDirectories::with_prefix("ovgu-canteen-gtk") {
                log::debug!("found cache directory for application");
                let history_path = match xdg.find_cache_file("history.json") {
                    Some(path) => path,
                    // if no cache is available, just skip
                    None => {
                        log::debug!("no history cache available for application");
                        break;
                    },
                };
                log::debug!("found history cache in {:?}", history_path);

                // this cannot fail, as xdg.find_cache_file makes sure the file exists
                let history_file = File::open(history_path).unwrap();
                let history: Vec<Canteen> = match serde_json::from_reader(history_file) {
                    Ok(history) => history,
                    // if parsing the cache fails, just skip
                    Err(e) => {
                        log::warn!("failed to parse cache: {:#}", e);
                        break;
                    },
                };

                for canteen in history {
                    canteen_cache.insert(canteen.description.clone(), canteen);
                }
            }

            log::debug!("finish loading cache");

            future::join_all(CANTEENS.iter().map(|canteen_desc| {
                let cached_canteen = canteen_cache.remove(canteen_desc);
                enclose! { (mut tx) async move {
                    let canteen_result = Self::load_canteen(cached_canteen, canteen_desc.clone()).await
                        .map(|mut canteen| {
                            canteen.days = canteen.days.drain(..)
                                // remove old menus
                                .filter(|day| day.date >= history_oldest)
                                .collect();
                            canteen
                        });

                    log::debug!("sending filtered canteen {:?} to UI", canteen_desc);
                    tx.send((canteen_desc.clone(), canteen_result)).await
                        .expect("Failed to commit downloaded canteen into UI component!");
                }}
            })).await;
        });

        let c = glib::MainContext::default();
        let fetch_reload_button = self.reload_button.clone();
        let fetch_canteen_components = Rc::clone(&self.canteen_components);
        let window_stack = SendWrapper::new(self.window_stack.clone());
        let window_error_label = SendWrapper::new(self.window_error_label.clone());
        c.spawn_local(enclose! { (rt) async move {
            // fetching parallel loaded canteens here and inserting
            // one canteen after another into the GUI.
            // TODO: render currently visible canteen first
            let mut canteen_cache = Vec::new();

            log::debug!("waiting for canteens to be downloaded...");
            while let Some((canteen_desc, canteen_result)) = rx.recv().await {
                log::debug!("canteen {:?} got downloaded", canteen_desc);
                if let Some(comp) = fetch_canteen_components.borrow_mut().get_mut(&canteen_desc) {
                    log::debug!("loading canteen {:?} into CanteenComponent", canteen_desc);
                    if let Some(canteen) = comp.load(canteen_result).await {
                        canteen_cache.push(canteen);
                    }
                } else {
                    log::error!("error: canteen {:?} not found in components list", canteen_desc);
                    window_stack.set_visible_child_name("window-error");
                    window_error_label.set_text(&format!(
                        "{}: canteen {:?} not found in components list",
                        t("error"),
                        canteen_desc,
                    ));
                }
            }

            log::debug!("finish loading canteens");

            fetch_reload_button.set_sensitive(true);

            if let Some(loaded) = loaded {
                log::debug!("notifying canteens loaded");
                loaded.notify();
            }

            rt.spawn(async move {
                log::debug!("write loaded canteens into history cache");
                // loop will only run once, used to abort early with break as ? is not available in scopes
                for xdg in xdg::BaseDirectories::with_prefix("ovgu-canteen-gtk") {
                    log::debug!("found cache directory for application");
                    let history_path = match xdg.place_cache_file("history.json") {
                        Ok(path) => path,
                        // if no cache is available, just skip
                        Err(e) => {
                            log::warn!("could not place history cache: {:#}", e);
                            break;
                        },
                    };
                    log::debug!("writing history cache to {:?}", history_path);

                    // this cannot fail, as xdg.find_cache_file makes sure the file exists
                    let history_file = File::create(history_path).unwrap();
                    if let Err(e) = serde_json::to_writer_pretty(history_file, &canteen_cache) {
                        log::warn!("failed to write history cache: {:#}", e);
                    };

                    log::debug!("finish writing history cache");
                }
            });
        }});
    }
}
