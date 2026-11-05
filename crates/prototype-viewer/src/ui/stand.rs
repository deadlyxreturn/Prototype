//! Firestorm's Stand / Stop flying button (panel_stand_stop_flying.xml):
//! while seated or flying, one button above the bottom bar does it, without
//! opening any window. Prototype's glossy pill.

use super::bars::BarAction;
use super::widgets::{self, ActionKind};
use crate::theme::Palette;
use egui::Vec2;

pub fn show(ctx: &egui::Context, p: &Palette, seated: bool, flying: bool) -> Option<BarAction> {
    if !seated && !flying {
        return None;
    }
    let screen = ctx.content_rect();
    let mut action = None;
    let shown = egui::Area::new(egui::Id::unique("stand_stop_flying"))
        .order(egui::Order::Middle)
        .pivot(egui::Align2::CENTER_BOTTOM)
        .fixed_pos(egui::pos2(screen.center().x, screen.bottom() - 58.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let size = Vec2::new(150.0, 38.0);
                if seated
                    && widgets::action_button(ui, p, "man-filled", prototype_i18n::tr!("common-stand-up"), ActionKind::Primary, size).clicked()
                {
                    action = Some(BarAction::StandUp);
                }
                if flying
                    && widgets::action_button(ui, p, "plane-tilt-filled", prototype_i18n::tr!("ui-bars-stop-flying"), ActionKind::Primary, size)
                        .clicked()
                {
                    action = Some(BarAction::ToggleFly);
                }
            });
        });
    // interface: left out of a "World + bars" video, like the windows
    widgets::note_window_rect(ctx, shown.response.rect);
    action
}
