//! Themes: flat, solid colours only (no gradients, no textures, no glow). Each theme is a
//! neutral surface ramp plus one restrained accent; the UI reads everything from `Pal`.

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureId, TextureOptions};

pub struct ThemeDef {
    pub id: &'static str,
    pub name: &'static str,
    pub dark: bool,
    /// window, sidebar, card, card under the pointer, popup
    surf: [u32; 5],
    /// hairline border, stronger border
    line: [u32; 2],
    /// primary, secondary, tertiary text
    ink: [u32; 3],
    pub accent: u32,
}

pub const THEMES: &[ThemeDef] = &[
    ThemeDef { id: "graphite", name: "Graphite", dark: true, surf: [0x111214, 0x16171a, 0x1a1b1f, 0x202126, 0x232429], line: [0x2a2b31, 0x3a3c44], ink: [0xececef, 0xa3a3ab, 0x70717a], accent: 0x4c8dff },
    ThemeDef { id: "carbon", name: "Carbon", dark: true, surf: [0x0b0b0c, 0x101011, 0x141416, 0x1a1a1d, 0x1c1c1f], line: [0x242427, 0x34343a], ink: [0xf2f2f3, 0xa1a1a6, 0x6c6c72], accent: 0xe8e8ea },
    ThemeDef { id: "slate", name: "Slate", dark: true, surf: [0x0f1318, 0x13181e, 0x171d24, 0x1c232b, 0x1f262e], line: [0x26303a, 0x35414d], ink: [0xe8edf2, 0x9aa6b2, 0x687482], accent: 0x2fb39b },
    ThemeDef { id: "midnight", name: "Midnight", dark: true, surf: [0x0e0f17, 0x12131d, 0x161724, 0x1b1d2b, 0x1e2030], line: [0x262838, 0x35374c], ink: [0xe9e9f2, 0xa0a1b5, 0x6d6e84], accent: 0x8b8cf8 },
    ThemeDef { id: "light", name: "Light", dark: false, surf: [0xf4f4f6, 0xeeeef1, 0xffffff, 0xf8f8fa, 0xffffff], line: [0xe1e1e6, 0xcbcbd3], ink: [0x18181b, 0x52525b, 0x8a8a93], accent: 0x2563eb },
];

/// Index of a saved theme name. Names of earlier themes map to their closest flat theme.
pub fn index(saved: &str) -> usize {
    let id = match saved {
        "aurora" | "ember" | "dark" | "" => "graphite",
        "oled" | "onyx" => "carbon",
        "tide" | "moss" => "slate",
        "orchid" => "midnight",
        "daybreak" => "light",
        s => s,
    };
    THEMES.iter().position(|t| t.id == id).unwrap_or(0)
}

pub fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// An opaque colour at the given opacity.
pub fn alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a.clamp(0.0, 1.0) * 255.0) as u8)
}

#[derive(Clone, Copy)]
pub struct Pal {
    pub dark: bool,
    pub bg: Color32,
    pub side: Color32,
    /// card fill
    pub panel: Color32,
    /// card fill under the pointer
    pub hover: Color32,
    /// popups, menus, toasts
    pub solid: Color32,
    pub border: Color32,
    pub border_hi: Color32,
    pub text: Color32,
    pub dim: Color32,
    pub muted: Color32,
    pub faint: Color32,
    /// switch track while off
    pub toggle: Color32,
    /// icon tile / quiet fill
    pub chip: Color32,
    pub ok: Color32,
    pub fail: Color32,
    pub run: Color32,
    pub acc: Color32,
    /// accent under the pointer
    pub acc_hi: Color32,
    /// faint accent wash (selected rows, focus rings)
    pub acc_soft: Color32,
    /// text on the accent
    pub on_acc: Color32,
    pub brave: TextureId,
}

pub struct Textures {
    brave: TextureHandle,
}

impl Textures {
    pub fn new(ctx: &egui::Context) -> Self {
        Textures { brave: ctx.load_texture("brave", brave_logo(), TextureOptions::LINEAR) }
    }
}

pub fn pal(i: usize, tex: &Textures) -> Pal {
    let t = &THEMES[i];
    let acc = hex(t.accent);
    let text = hex(t.ink[0]);
    // a near-white accent (Carbon) carries dark text, every other accent white text
    let light_acc = acc.r() as u32 + acc.g() as u32 + acc.b() as u32 > 600;
    Pal {
        dark: t.dark,
        bg: hex(t.surf[0]),
        side: hex(t.surf[1]),
        panel: hex(t.surf[2]),
        hover: hex(t.surf[3]),
        solid: hex(t.surf[4]),
        border: hex(t.line[0]),
        border_hi: hex(t.line[1]),
        text,
        dim: hex(t.ink[1]),
        muted: hex(t.ink[2]),
        faint: hex(t.line[1]),
        toggle: if t.dark { hex(t.line[1]) } else { hex(0xc9c9d1) },
        chip: if t.dark { hex(t.surf[3]) } else { hex(0xf0f0f3) },
        ok: hex(if t.dark { 0x3fb97a } else { 0x15803d }),
        fail: hex(if t.dark { 0xef5b5b } else { 0xdc2626 }),
        run: hex(if t.dark { 0xd9a23b } else { 0xb45309 }),
        acc,
        acc_hi: if t.dark { acc.lerp_to_gamma(Color32::WHITE, 0.12) } else { acc.lerp_to_gamma(Color32::BLACK, 0.10) },
        acc_soft: alpha(acc, if t.dark { 0.14 } else { 0.10 }),
        on_acc: if light_acc { hex(0x111214) } else { Color32::WHITE },
        brave: tex.brave.id(),
    }
}

/// Brave's own logo, cropped from the Brave install on the PC this was built on.
fn brave_logo() -> ColorImage {
    let mut reader = png::Decoder::new(&include_bytes!("../data/icons/brave.png")[..]).read_info().expect("brave.png");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("brave.png");
    ColorImage::from_rgba_unmultiplied([info.width as usize, info.height as usize], &buf[..info.buffer_size()])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_theme_names_still_resolve() {
        assert_eq!(THEMES[index("oled")].id, "carbon");
        assert_eq!(THEMES[index("onyx")].id, "carbon");
        assert_eq!(THEMES[index("daybreak")].id, "light");
        assert_eq!(THEMES[index("aurora")].id, "graphite");
        assert_eq!(index("dark"), 0);
        assert_eq!(index(""), 0);
        assert_eq!(THEMES[index("slate")].id, "slate");
    }
}
