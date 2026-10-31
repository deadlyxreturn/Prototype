//! Prototype's brand art: the login background (the glass bear in the
//! code rain), the PROTOTYPE wordmark and the bear's head. Decoded once,
//! off the main thread, the first time a screen asks for them; until then
//! the callers draw without them (the prototype background, the text title).

use egui::{Context, TextureHandle, TextureOptions};
use std::sync::OnceLock;

const BG_JPG: &[u8] = include_bytes!("../../../../assets/branding/prototype-login-bg.jpg");
const WORDMARK_PNG: &[u8] = include_bytes!("../../../../assets/branding/prototype-wordmark.png");
const BEAR_PNG: &[u8] = include_bytes!("../../../../assets/branding/prototype-bear.png");
const WAVE_JPG: &[u8] = include_bytes!("../../../../assets/branding/prototype-login-bg-wave.jpg");

/// The login background chosen in Preferences › Appearance (the setting
/// `prototype_login_bg`), read by `background`.
static CHOICE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn set_background_choice(choice: u8) {
    CHOICE.store(choice, std::sync::atomic::Ordering::Relaxed);
}

/// The user's own login picture, in the config folder.
pub fn custom_background_path() -> Option<std::path::PathBuf> {
    let dir = crate::settings::config_dir();
    ["login-background.png", "login-background.jpg", "login-background.jpeg"]
        .iter()
        .map(|name| dir.join(name))
        .find(|path| path.is_file())
}

/// The chosen login background's texture (the bear until the others are
/// decoded, or when the user's own picture is missing).
pub fn background(ctx: &Context) -> Option<TextureHandle> {
    let art = match CHOICE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => Art::Wave,
        2 => Art::Custom,
        _ => Art::Background,
    };
    texture(ctx, art).or_else(|| texture(ctx, Art::Background))
}

#[derive(Clone, Copy)]
pub enum Art {
    Background,
    Wordmark,
    Bear,
    Wave,
    Custom,
}

impl Art {
    fn bytes(self) -> std::borrow::Cow<'static, [u8]> {
        match self {
            Art::Background => BG_JPG.into(),
            Art::Wordmark => WORDMARK_PNG.into(),
            Art::Bear => BEAR_PNG.into(),
            Art::Wave => WAVE_JPG.into(),
            Art::Custom => custom_background_path()
                .and_then(|path| std::fs::read(path).map_err(|e| log::warn!("login background: {e}")).ok())
                .unwrap_or_default()
                .into(),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Art::Background => "prototype-bg",
            Art::Wordmark => "prototype-wordmark",
            Art::Bear => "prototype-bear",
            Art::Wave => "prototype-wave",
            Art::Custom => "prototype-custom-bg",
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}

type Decoded = [OnceLock<Option<egui::ColorImage>>; 5];
static DECODED: OnceLock<Decoded> = OnceLock::new();
static STARTED: OnceLock<()> = OnceLock::new();

fn decoded() -> &'static Decoded {
    DECODED.get_or_init(|| std::array::from_fn(|_| OnceLock::new()))
}

/// Starts the decoding (once) and asks egui to look again shortly.
fn start(ctx: &Context) {
    if STARTED.set(()).is_ok() {
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            for art in [Art::Wordmark, Art::Bear, Art::Background, Art::Wave, Art::Custom] {
                let bytes = art.bytes();
                if bytes.is_empty() {
                    let _ = decoded()[art.index()].set(None);
                    continue;
                }
                let image = match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        Some(egui::ColorImage::from_rgba_unmultiplied(
                            [rgba.width() as usize, rgba.height() as usize],
                            rgba.as_raw(),
                        ))
                    }
                    Err(e) => {
                        log::warn!("{} decode failed: {e}", art.name());
                        None
                    }
                };
                let _ = decoded()[art.index()].set(image);
                ctx.request_repaint();
            }
        });
    }
}

/// The texture of `art`, once decoded (None the first frames).
pub fn texture(ctx: &Context, art: Art) -> Option<TextureHandle> {
    let id = egui::Id::new(("prototype_art", art.index()));
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return Some(t);
    }
    start(ctx);
    let image = decoded()[art.index()].get()?.clone()?;
    let t = ctx.load_texture(art.name(), image, TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    Some(t)
}

/// Paints `t` to cover `rect` (cropped, never stretched), like CSS
/// `background-size: cover`.
pub fn paint_cover(painter: &egui::Painter, t: &TextureHandle, rect: egui::Rect, tint: egui::Color32) {
    let [w, h] = t.size();
    let (iw, ih) = (w as f32, h as f32);
    let scale = (rect.width() / iw).max(rect.height() / ih);
    let (sw, sh) = (rect.width() / (iw * scale), rect.height() / (ih * scale));
    // keep the bear (right third) in view on narrow windows: anchor right
    let u0 = (1.0 - sw).max(0.0) * 0.75;
    let v0 = (1.0 - sh).max(0.0) * 0.5;
    let uv = egui::Rect::from_min_max(egui::pos2(u0, v0), egui::pos2(u0 + sw, v0 + sh));
    painter.image(t.id(), rect, uv, tint);
}

/// The size `t` takes at `width` points wide.
pub fn size_at_width(t: &TextureHandle, width: f32) -> egui::Vec2 {
    let [w, h] = t.size();
    egui::vec2(width, width * h as f32 / w.max(1) as f32)
}
