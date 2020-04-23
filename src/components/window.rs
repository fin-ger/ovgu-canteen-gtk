use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use anyhow::{bail, Context, Result};
use cargo_author::Author;
use gio::prelude::*;
use gio::SimpleAction;
use gtk::prelude::*;
use gtk::{
    AboutDialog, Box, Builder, Button, ButtonRole, Label, MenuButton, ModelButtonBuilder, Stack,
    Window,
};
use ovgu_canteen::{Canteen, CanteenDescription};
use tokio::runtime::Handle;
use tokio::sync::mpsc::channel;
use send_wrapper::SendWrapper;

use crate::components::{get, CanteenComponent, GLADE};
use crate::util::enclose;

#[derive(Debug)]
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
}

impl WindowComponent {
    pub fn new(rt: &Handle, app: &gtk::Application) -> Result<()> {
        let builder = Builder::new_from_string(GLADE);

        let mut canteens = vec![
            CanteenDescription::UniCampusLowerHall,
            CanteenDescription::UniCampusUpperHall,
            CanteenDescription::Kellercafe,
            CanteenDescription::Herrenkrug,
            CanteenDescription::Stendal,
            CanteenDescription::Wernigerode,
            CanteenDescription::DomCafeteHalberstadt,
        ];
        let window: Window = get!(&builder, "window")?;
        let window_stack: Stack = get!(&builder, "window-stack")?;
        let window_error_label: Label = get!(&builder, "window-error-label")?;
        let canteens_stack: Stack = get!(&builder, "canteens-stack")?;
        let canteens_menu: Box = get!(&builder, "canteens-menu")?;
        let canteen_label: Label = get!(&builder, "canteen-label")?;
        let canteen_menu_button: MenuButton = get!(&builder, "canteen-menu-button")?;
        let about_dialog: AboutDialog = get!(&builder, "about")?;
        let about_button: Button = get!(&builder, "about-btn")?;
        let options_button: MenuButton = get!(&builder, "options-button")?;
        let reload_button: Button = get!(&builder, "reload-button")?;

        window.set_application(Some(app));
        window.set_icon_name(Some("ovgu-canteen32"));
        about_dialog.set_logo_icon_name(Some("ovgu-canteen128"));

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
        about_button.connect_clicked(move |_btn| {
            if let Some(popover) = options_button.get_popover() {
                popover.popdown();
            }

            about_dialog.run();
            about_dialog.hide();
        });

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

            canteens_stack_handle.set_visible_child_name(canteen_name);
            canteen_label_handle.set_text(canteen_name);
        });
        app.add_action(&canteen_selected_action);

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
        };

        let mut canteen_components_borrow = comp.canteen_components.borrow_mut();
        for desc in canteens.drain(..) {
            canteen_components_borrow.insert(
                desc.clone(),
                CanteenComponent::new(&desc, &comp).context("Failed to create canteen!")?,
            );
        }
        drop(canteen_components_borrow);

        comp.load(rt);
        comp.reload_button.clone().connect_clicked(enclose! { (rt) move |_btn| {
            comp.load(&rt);
        }});

        Ok(())
    }

    pub fn add_canteen(&self, canteen_stack: &Stack, canteen_name: &'static str) -> Result<()> {
        self.canteens_stack.add_named(canteen_stack, canteen_name);

        let model_btn = ModelButtonBuilder::new()
            .visible(true)
            .text(canteen_name)
            .can_focus(false)
            .action_name("app.canteen-selected")
            .action_target(&canteen_name.to_variant())
            .role(ButtonRole::Radio)
            .build();

        self.canteens_menu.pack_start(&model_btn, false, true, 0);

        Ok(())
    }

    pub fn load(&self, rt: &Handle) {
        self.reload_button.set_sensitive(false);
        self.window_stack.set_visible_child_name("canteens-stack");

        // canteens are downloaded in parallel here,
        // but in order for one canteen to show up in a batch
        // we are using an mpsc channel to put the parallel loaded canteens
        // in an order which is later sequentially inserted into the GUI.
        let (tx, mut rx) = channel(self.canteen_components.borrow().len());
        for (canteen_desc, _comp) in self.canteen_components.borrow().iter() {
            let mut tx = tx.clone();
            rt.spawn(enclose! { (canteen_desc) async move {
                let canteen = (enclose! { (canteen_desc) || async move {
                    if cfg!(feature = "test-with-local-files") {
                        use std::fs::File;

                        let file = File::open("data/canteens.json")
                            .context("'data/canteens.json' not found!")?;
                        let mut canteens: Vec<Canteen> = serde_json::from_reader(&file)
                            .context("Could not parse 'data/cateens.json'")?;
                        let canteen = canteens
                            .drain(..)
                            .find(|c| c.description == canteen_desc)
                            .context("Canteen not found!")?;
                        Ok(canteen)
                    } else {
                        failure::ResultExt::compat(Canteen::new(canteen_desc.clone()).await)
                            .context("Failed to fetch canteen")
                    }
                }})().await;
                tx.send((canteen_desc, canteen)).await
                    .expect("Failed to commit downloaded canteen into UI component!");
            }});
        }

        let c = glib::MainContext::default();
        let fetch_reload_button = self.reload_button.clone();
        let fetch_canteen_components = Rc::clone(&self.canteen_components);
        let window_stack = SendWrapper::new(self.window_stack.clone());
        let window_error_label = SendWrapper::new(self.window_error_label.clone());
        c.spawn_local(async move {
            // fetching parallel loaded canteens here and inserting
            // one canteen after another into the GUI.
            // TODO: render currently visible canteen first
            while let Some((desc, canteen)) = rx.recv().await {
                if let Some(comp) = fetch_canteen_components.borrow_mut().get_mut(&desc) {
                    comp.load(canteen).await;
                } else {
                    window_stack.set_visible_child_name("window-error");
                    window_error_label.set_text(&format!("error: canteen {:?} not found in components list", desc));
                }
            }

            fetch_reload_button.set_sensitive(true);
        });
    }
}
