//! Shared login information, community links, version footer and the
//! preferences button.
use super::{LoginAction, LoginForm};
use crate::theme::Palette;
use crate::ui::{
    icons::Icons,
    news::LoginInfo,
    widgets::{self, ActionKind},
};
use egui::{Context, Id, Rect, RichText, Vec2};

pub fn sidebar(ctx: &Context, p: &Palette, icons: &Icons, info: &LoginInfo, card: Rect, width: f32, lowest: f32) -> bool {
    let left = ctx.content_rect().right() - 32.0 - width;
    egui::Area::new(Id::unique("login_news"))
        .fixed_pos(egui::pos2(left, card.top()))
        .show(ctx, |ui| {
            ui.set_width(width);
            super::news_panel(ui, p, icons, info, card.height().min(lowest - card.top()).max(120.0));
        });
    egui::Area::new(Id::unique("login_version"))
        .pivot(egui::Align2::LEFT_BOTTOM)
        .fixed_pos(egui::pos2(left, card.top() - 14.0))
        .show(ctx, |ui| {
            ui.set_width(width);
            super::version_panel(ui, p, icons, info)
        })
        .inner
}

fn version(ui: &mut egui::Ui, p: &Palette) {
    let label = prototype_i18n::tr!("ui-login-prototype-viewer-v", value0 = prototype_version::version().to_string());
    // Prototype: the mockup's status pill of frosted glass
    egui::Frame::new()
        .fill(p.field.gamma_multiply(0.55))
        .stroke(egui::Stroke::new(1.0, p.field))
        .corner_radius(14) // theme-guard: content — Prototype's status pill
        .inner_margin(egui::Margin::symmetric(14, 7))
        .show(ui, |ui| ui.label(RichText::new(label).size(12.0).monospace().color(p.muted)));
}

pub fn footer(ctx: &Context, p: &Palette, icons: &Icons, info: &LoginInfo, sidebar_visible: bool, form: &mut LoginForm) -> LoginAction {
    let mut action = LoginAction::None;
    let screen = ctx.content_rect();
    if form.safe_graphics.is_some() || form.driver_notice.is_some() {
        super::driver_notice::column(ctx, |ui, width| {
            if let Some(state) = form.safe_graphics
                && let Some(choice) = super::safe_graphics_notice::card(ui, p, icons, state, width)
            {
                action = LoginAction::SafeGraphics(choice);
            }
            if let Some(notice) = &form.driver_notice
                && super::driver_notice::card(ui, p, icons, notice, width)
            {
                action = LoginAction::DismissDriverNotice;
            }
        });
    }
    // Prototype: no community corner; the version line says where to go
    let _ = (icons, sidebar_visible, &mut *form);
    if screen.width() < 1100.0 {
        egui::Area::new(Id::unique("login_footer_version"))
            .anchor(egui::Align2::RIGHT_BOTTOM, Vec2::new(-25.0, -59.0))
            .show(ctx, |ui| version(ui, p));
    }
    // The preferences before login (Firestorm opens them from the menu bar of
    // its login screen, menu_login.xml): one cogwheel in the top right corner.
    egui::Area::new(Id::unique("login_settings"))
        .anchor(egui::Align2::RIGHT_TOP, Vec2::new(-25.0, 20.0))
        .show(ctx, |ui| {
            if widgets::action_button(ui, p, "settings-filled", "", ActionKind::Flat, Vec2::splat(30.0))
                .on_hover_text(prototype_i18n::tr!("common-preferences"))
                .clicked()
            {
                action = LoginAction::Preferences;
            }
        });
    // Left of it, the way to a larger, more readable interface before the
    // first login: Preferences › Accessibility.
    egui::Area::new(Id::unique("login_accessibility"))
        .anchor(egui::Align2::RIGHT_TOP, Vec2::new(-63.0, 20.0))
        .show(ctx, |ui| {
            if widgets::action_button(ui, p, "accessible-filled", "", ActionKind::Flat, Vec2::splat(30.0))
                .on_hover_text(prototype_i18n::tr!("login-accessibility"))
                .clicked()
            {
                action = LoginAction::Accessibility;
            }
        });
    if screen.width() >= 1100.0 {
        egui::Area::new(Id::unique("login_quit"))
            .anchor(egui::Align2::RIGHT_BOTTOM, Vec2::new(-25.0, -20.0))
            .show(ctx, |ui| version(ui, p));
    }
    if form.show_info {
        let id = Id::unique("accounts_news_dialog");
        prototype_render::blurred_backdrop(ctx, egui::Modal::default_area(id).layer());
        let response = widgets::Modal::new(id).transparent_backdrop().show(ctx, |ui| {
            ui.set_width(380.0);
            ui.spacing_mut().item_spacing.y = 8.0;
            // Prototype: the bear and the name, like Preferences' title row
            ui.horizontal(|ui| {
                if let Some(bear) = super::super::prototype_art::texture(ctx, super::super::prototype_art::Art::Bear) {
                    let size = super::super::prototype_art::size_at_width(&bear, 30.0);
                    ui.add(egui::Image::new(&bear).fit_to_exact_size(size));
                }
                ui.label(RichText::new(prototype_i18n::tr!("ui-login-news")).size(20.0).color(p.ink));
            });
            if super::version_panel(ui, p, icons, info) {
                action = LoginAction::CheckUpdates;
            }
            ui.add_space(8.0);
            super::news_panel(ui, p, icons, info, 235.0);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::action_button(ui, p, "", prototype_i18n::tr!("common-close"), ActionKind::Primary, Vec2::new(130.0, 40.0)).clicked() {
                    form.show_info = false;
                }
            });
        });
        if response.should_close() {
            form.show_info = false;
        }
    }
    action
}
