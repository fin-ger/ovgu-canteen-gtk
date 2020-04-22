use anyhow::Result;
use gtk::prelude::*;
use gtk::Label;

#[derive(Debug)]
pub struct BadgeComponent {
    pub label: Label,
}

#[derive(Debug)]
pub struct LiteBadgeComponent {
    pub label: Label,
}

impl BadgeComponent {
    pub async fn new() -> Result<Self> {
        let label: Label = Label::new(None);
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
        let label: Label = Label::new(None);
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
