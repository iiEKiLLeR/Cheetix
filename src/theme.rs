//! Themes: a near-black ground with a slow, animated aurora in the theme's colour (light
//! blades that drift and breathe, film grain and a vignette), glass-like translucent surfaces
//! and one accent. Slate and Light are flat. The UI reads everything from `Pal`.

use eframe::egui::{self, pos2, vec2, Color32, ColorImage, Mesh, Painter, Rect, Shape, TextureHandle, TextureId, TextureOptions};
use std::f32::consts::{PI, TAU};

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
    /// aurora colours: core, middle, tip (none: flat background)
    aura: Option<[u32; 3]>,
}

const NIGHT: [u32; 5] = [0x07080a, 0x0b0c0f, 0x111216, 0x17181d, 0x141519];
const NIGHT_LINE: [u32; 2] = [0x25262c, 0x36373f];
const NIGHT_INK: [u32; 3] = [0xffffff, 0x9c9c9d, 0x6b6b70];

pub const THEMES: &[ThemeDef] = &[
    ThemeDef { id: "cheetah", name: "Cheetah", dark: true, surf: NIGHT, line: NIGHT_LINE, ink: NIGHT_INK, accent: 0xff6a00, aura: Some([0xff3a1a, 0xff6a00, 0xffb347]) },
    ThemeDef { id: "mono", name: "Mono", dark: true, surf: NIGHT, line: NIGHT_LINE, ink: NIGHT_INK, accent: 0xe8e8ea, aura: Some([0x4a4a50, 0x8a8a92, 0xc8c8d0]) },
    ThemeDef { id: "violet", name: "Violet", dark: true, surf: NIGHT, line: NIGHT_LINE, ink: NIGHT_INK, accent: 0xa855f7, aura: Some([0x6d28d9, 0xa855f7, 0xf0abfc]) },
    ThemeDef { id: "slate", name: "Slate", dark: true, surf: [0x0f1318, 0x13181e, 0x171d24, 0x1c232b, 0x1f262e], line: [0x26303a, 0x35414d], ink: [0xe8edf2, 0x9aa6b2, 0x687482], accent: 0x2fb39b, aura: None },
    ThemeDef { id: "light", name: "Light", dark: false, surf: [0xf4f4f6, 0xeeeef1, 0xffffff, 0xf8f8fa, 0xffffff], line: [0xe1e1e6, 0xcbcbd3], ink: [0x18181b, 0x52525b, 0x8a8a93], accent: 0x2563eb, aura: None },
];

/// Index of a saved theme name. Names of earlier themes map to their closest flat theme.
pub fn index(saved: &str) -> usize {
    let id = match saved {
        "graphite" | "aurora" | "ember" | "dark" | "" => "cheetah",
        "carbon" | "oled" | "onyx" => "mono",
        "midnight" | "orchid" => "violet",
        "tide" | "moss" => "slate",
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
    /// aurora colours, when the theme has one
    pub aura: Option<[Color32; 3]>,
    pub grain: TextureId,
    pub logo: TextureId,
}

pub struct Textures {
    brave: TextureHandle,
    grain: TextureHandle,
    logo: TextureHandle,
}

impl Textures {
    pub fn new(ctx: &egui::Context) -> Self {
        Textures { brave: ctx.load_texture("brave", png_image(include_bytes!("../data/icons/brave.png")), TextureOptions::LINEAR), logo: ctx.load_texture("logo", png_image(include_bytes!("../assets/cheetix-256.png")), TextureOptions::LINEAR), grain: ctx.load_texture("grain", grain(), TextureOptions::LINEAR_REPEAT) }
    }
}

pub fn pal(i: usize, tex: &Textures) -> Pal {
    let t = &THEMES[i];
    let acc = hex(t.accent);
    let text = hex(t.ink[0]);
    // a near-white accent (Carbon) carries dark text, every other accent white text
    let light_acc = acc.r() as u32 + acc.g() as u32 + acc.b() as u32 > 600;
    // over an aurora, surfaces are dark glass so the light shows through
    let glass = |c: u32, a: f32| if t.aura.is_some() { alpha(hex(c), a) } else { hex(c) };
    Pal {
        dark: t.dark,
        bg: hex(t.surf[0]),
        side: glass(t.surf[1], 0.55),
        panel: glass(t.surf[2], 0.66),
        hover: glass(t.surf[3], 0.78),
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
        aura: t.aura.map(|c| c.map(hex)),
        grain: tex.grain.id(),
        logo: tex.logo.id(),
    }
}

/// The window ground plus, for aurora themes, the animated light. `t`: seconds since start.
/// `k`: size of `r` relative to a 1280-wide window (theme previews pass a small one).
pub fn paint_ground(painter: &Painter, r: Rect, p: &Pal, t: f32, k: f32) {
    painter.rect_filled(r, 0.0, p.bg);
    let Some(c) = p.aura else { return };
    // (centre x, centre y, angle deg, length x diagonal, half-width px, period s, phase)
    const BLADES: [(f32, f32, f32, f32, f32, f32, f32); 5] = [
        (0.78, 0.10, 148.0, 0.95, 210.0, 19.0, 0.0),
        (0.95, 0.42, 132.0, 0.80, 150.0, 15.0, 1.7),
        (0.60, 0.00, 160.0, 0.70, 120.0, 22.0, 3.1),
        (0.12, 0.98, -24.0, 0.70, 170.0, 17.0, 4.4),
        (0.42, 0.62, 150.0, 0.55, 90.0, 21.0, 2.3),
    ];
    let diag = r.width().hypot(r.height());
    for (cx, cy, ang, len, w, per, ph) in BLADES {
        // drift (translate + slight rotate) and breathe (opacity + scale); phase 0 is full bloom
        let s = (t / per * TAU + ph).cos();
        let ctr = pos2(r.left() + r.width() * cx, r.top() + r.height() * cy) + vec2(46.0, 28.0) * s * k;
        let a = (ang + 4.0 * s).to_radians();
        let (dir, nrm) = (vec2(a.cos(), a.sin()), vec2(-a.sin(), a.cos()));
        let (len, w, glow) = (diag * len * (0.94 + 0.06 * s), w * k * (0.9 + 0.1 * s), 0.72 + 0.28 * s);
        let mut m = Mesh::default();
        // soft focus across the blade, colour ramp core -> middle -> tip along it
        const ACROSS: [(f32, f32); 7] = [(-1.0, 0.0), (-0.62, 0.18), (-0.3, 0.62), (0.0, 1.0), (0.3, 0.62), (0.62, 0.18), (1.0, 0.0)];
        const ALONG: u32 = 14;
        for i in 0..=ALONG {
            let u = i as f32 / ALONG as f32;
            let col = if u < 0.5 { c[0].lerp_to_gamma(c[1], u * 2.0) } else { c[1].lerp_to_gamma(c[2], u * 2.0 - 1.0) };
            let fade = (u * PI).sin().powf(1.4) * glow * 0.8;
            for (v, wt) in ACROSS {
                let pt = ctr + dir * (u - 0.5) * len + nrm * v * w * (1.15 - 0.5 * u);
                let f = wt * fade;
                // alpha 0: additive light on the dark ground
                m.colored_vertex(pt, Color32::from_rgba_premultiplied((col.r() as f32 * f) as u8, (col.g() as f32 * f) as u8, (col.b() as f32 * f) as u8, 0));
            }
        }
        for i in 0..ALONG {
            for j in 0..6 {
                let a = i * 7 + j;
                m.add_triangle(a, a + 1, a + 7);
                m.add_triangle(a + 1, a + 8, a + 7);
            }
        }
        painter.add(Shape::mesh(m));
    }
    // film grain, so it never reads as a flat gradient
    let mut g = Mesh::with_texture(p.grain);
    g.add_rect_with_uv(r, Rect::from_min_max(pos2(0.0, 0.0), pos2(r.width() / 128.0, r.height() / 128.0)), Color32::WHITE);
    painter.add(Shape::mesh(g));
    // vignette: darker toward the edges
    let mut v = Mesh::default();
    const N: u32 = 10;
    for y in 0..=N {
        for x in 0..=N {
            let (fx, fy) = (x as f32 / N as f32, y as f32 / N as f32);
            let d = ((fx - 0.5).powi(2) + (fy - 0.5).powi(2)).sqrt() * 2.0;
            v.colored_vertex(pos2(r.left() + r.width() * fx, r.top() + r.height() * fy), Color32::from_black_alpha((((d - 0.55) / 0.85).clamp(0.0, 1.0).powi(2) * 170.0) as u8));
        }
    }
    for y in 0..N {
        for x in 0..N {
            let a = y * (N + 1) + x;
            v.add_triangle(a, a + 1, a + N + 1);
            v.add_triangle(a + 1, a + N + 2, a + N + 1);
        }
    }
    painter.add(Shape::mesh(v));
}

/// 128x128 tile of faint monochrome noise.
fn grain() -> ColorImage {
    let mut x: u32 = 0x9e37_79b9;
    let pixels = (0..128 * 128)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let l = (x & 0xff) as u8;
            Color32::from_rgba_unmultiplied(l, l, l, 3)
        })
        .collect();
    ColorImage { size: [128, 128], pixels }
}

/// An RGBA PNG built into the exe (the app logo; Brave's logo, from a Brave install).
fn png_image(bytes: &[u8]) -> ColorImage {
    let mut reader = png::Decoder::new(bytes).read_info().expect("png");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("png");
    ColorImage::from_rgba_unmultiplied([info.width as usize, info.height as usize], &buf[..info.buffer_size()])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_theme_names_still_resolve() {
        assert_eq!(THEMES[index("oled")].id, "mono");
        assert_eq!(THEMES[index("carbon")].id, "mono");
        assert_eq!(THEMES[index("daybreak")].id, "light");
        assert_eq!(THEMES[index("graphite")].id, "cheetah");
        assert_eq!(THEMES[index("midnight")].id, "violet");
        assert_eq!(index("dark"), 0);
        assert_eq!(index(""), 0);
        assert_eq!(THEMES[index("slate")].id, "slate");
    }
}
