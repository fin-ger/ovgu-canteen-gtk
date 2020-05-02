use anyhow::Result;
use gtk::prelude::*;
use gtk::{Label, Image, IconSize};

pub struct BadgeComponent {
    label: Label,
}

pub struct LiteBadgeComponent {
    label: Label,
}

pub struct SymbolComponent {
    image: Image,
}

impl BadgeComponent {
    pub async fn new() -> Result<Self> {
        let label = Label::new(None);
        let context = label.get_style_context();
        context.add_class("badge");
        label.set_selectable(true);
        label.set_line_wrap(false);
        label.set_visible(true);

        Ok(Self { label })
    }

    pub const fn root_widget(&self) -> &Label {
        &self.label
    }

    pub async fn load(&self, text: &str) {
        self.label.set_text(text);
    }
}

impl LiteBadgeComponent {
    pub async fn new() -> Result<Self> {
        let label = Label::new(None);
        let context = label.get_style_context();
        context.add_class("badge-lite");
        label.set_selectable(true);
        label.set_line_wrap(false);
        label.set_visible(true);

        Ok(Self { label })
    }

    pub const fn root_widget(&self) -> &Label {
        &self.label
    }

    pub async fn load(&self, text: &str) {
        self.label.set_text(text);
    }
}

impl SymbolComponent {
    pub async fn new() -> Result<Self> {
        let image = Image::new();

        image.set_visible(true);

        Ok(Self { image })
    }

    pub const fn root_widget(&self) -> &Image {
        &self.image
    }

    pub async fn load(&self, name: &str, tooltip: &str) {
        self.image.set_from_icon_name(Some(name), IconSize::LargeToolbar);
        self.image.set_tooltip_text(Some(tooltip));
    }
}
