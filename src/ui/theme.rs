//! Heliox theme & styling system (in pure Rust with NSH).

use nsh_core::{
    animation::{Curve, EaseMode},
    element::{
        AdaptiveColor, BoxShadow, Easing, Interactive, Rgba, Spring, StyleRefinement, Styled,
        Timing, rgb, rgba, set_color_scheme,
    },
    geometry::px,
    platform::Appearance,
    prelude::{ComponentBuilder, global, reduce_motion},
};

use crate::{model::Config, save_config};

pub const THEMES: [&str; 3] = ["System", "Light", "Dark"];

pub fn apply_theme(name: &str) {
    let appearance = match name {
        "System" => Appearance::System,
        "Light" | "Default Light" => Appearance::Light,
        _ => Appearance::Dark,
    };
    set_color_scheme(appearance);
}

pub fn select_theme(name: impl Into<String>) {
    let name = name.into();
    global::<Config>().update(|config| config.theme_name = name.clone());
    apply_theme(&name);
    save_config(&global::<Config>().get_untracked());
}

pub trait AppRadius: Styled + Sized {
    fn app_radius(self, factor: f32) -> Self {
        self.rounded(move || px(global::<Config>().get().corner_radius * factor))
    }
}

impl<T: Styled> AppRadius for T {}

/// `#rrggbb` at opacity `alpha` (CSS's `rgba(r, g, b, alpha)`).
pub const fn rgb_a(hex: u32, alpha: f32) -> Rgba {
    rgba(hex << 8 | (alpha * 255.0 + 0.5) as u32)
}

/// A color for each appearance: `light`, `dark`.
pub const fn tone(light: Rgba, dark: Rgba) -> AdaptiveColor {
    AdaptiveColor {
        light: light.to_hsla(),
        dark: dark.to_hsla(),
    }
}

pub mod colors {
    use super::{AdaptiveColor, rgb, tone};

    pub const BACKGROUND: AdaptiveColor = tone(rgb(0xf4f4f2), rgb(0x101114));
    pub const SURFACE: AdaptiveColor = tone(rgb(0xffffff), rgb(0x17191d));
    pub const INSET: AdaptiveColor = tone(rgb(0xeeeee9), rgb(0x0e0f12));
    pub const INPUT: AdaptiveColor = tone(rgb(0xffffff), rgb(0x0e0f12));
    pub const INPUT_TEXT: AdaptiveColor = tone(rgb(0x16171a), rgb(0xeef0f4));
    pub const INPUT_MUTED: AdaptiveColor = tone(rgb(0x62666d), rgb(0x8f939b));
    pub const RAISED: AdaptiveColor = tone(rgb(0xfbfbf9), rgb(0x222429));
    pub const BORDER: AdaptiveColor = tone(rgb(0xd8d8d1), rgb(0x2b2e35));
    pub const LINE: AdaptiveColor = tone(rgb(0xe4e4de), rgb(0x22252a));
    pub const TEXT: AdaptiveColor = tone(rgb(0x16171a), rgb(0xeef0f4));
    pub const MUTED: AdaptiveColor = tone(rgb(0x5f636a), rgb(0x8f939b));
    pub const FAINT: AdaptiveColor = tone(rgb(0x9a9ea4), rgb(0x5c6068));
    pub const ACCENT: AdaptiveColor = tone(rgb(0x4a840a), rgb(0xc8f26d));
    pub const ACCENT_FILL: AdaptiveColor = tone(rgb(0xc8f26d), rgb(0xc8f26d));
    pub const ON_ACCENT: AdaptiveColor = tone(rgb(0x13160c), rgb(0x13160c));
    pub const LIGHT: AdaptiveColor = tone(rgb(0x16171a), rgb(0xeef0f2));
    pub const ON_LIGHT: AdaptiveColor = tone(rgb(0xffffff), rgb(0x111214));
    pub const TRACK: AdaptiveColor = tone(rgb(0xe2e2dd), rgb(0x24262a));
    pub const KNOB: AdaptiveColor = tone(rgb(0xffffff), rgb(0x8d9198));
    pub const DANGER: AdaptiveColor = tone(rgb(0xd43a4a), rgb(0xff6170));
    pub const WARN: AdaptiveColor = tone(rgb(0xb26a00), rgb(0xffb547));
    pub const UP: AdaptiveColor = tone(rgb(0x138a5a), rgb(0x45d49a));
    pub const DOWN: AdaptiveColor = tone(rgb(0xd43a4a), rgb(0xff6170));
}

pub mod tints {
    use super::{AdaptiveColor, rgb_a, tone};

    pub const ACCENT: AdaptiveColor = tone(rgb_a(0x7caa14, 0.14), rgb_a(0xc8f26d, 0.14));
    pub const DANGER: AdaptiveColor = tone(rgb_a(0xd43a4a, 0.10), rgb_a(0xff6170, 0.16));
    pub const WARN: AdaptiveColor = tone(rgb_a(0xb26a00, 0.11), rgb_a(0xffb547, 0.16));
}

pub mod radius {
    use nsh_core::geometry::{Points, px};

    pub const SM: Points = px(4.0);
    pub const MD: Points = px(8.0);
    pub const LG: Points = px(12.0);
    pub const XL: Points = px(16.0);
    pub const FULL: Points = px(9999.0);
}

pub mod motion {
    use super::{Curve, EaseMode, Easing, Spring, Timing};

    pub const PRESS: Spring = Spring {
        stiffness: 520.0,
        damping: 34.0,
        mass: 1.0,
        velocity: None,
        frame_rate: None,
    };

    pub const SETTLE: Spring = Spring {
        stiffness: 320.0,
        damping: 30.0,
        mass: 1.0,
        velocity: None,
        frame_rate: None,
    };

    pub const ENTER: Timing = Timing::new(420.0, Easing::Curve(Curve::Cubic, EaseMode::Out));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    Card,
    Inset,
    Raised,
    Light,
    Accent,
    Track,
    Knob,
    Pill,
}

const fn drop(y: f32, blur: f32, color: Rgba) -> BoxShadow {
    BoxShadow {
        offset_x: px(0.0),
        offset_y: px(y),
        blur: px(blur),
        spread: px(0.0),
        color: color.to_packed(),
        inset: false,
    }
}

const fn inner(y: f32, blur: f32, spread: f32, color: Rgba) -> BoxShadow {
    BoxShadow {
        offset_x: px(0.0),
        offset_y: px(y),
        blur: px(blur),
        spread: px(spread),
        color: color.to_packed(),
        inset: true,
    }
}

pub trait Finished: Styled + Sized {
    fn finish(self, finish: Finish) -> Self {
        self.finish_fill(finish).finish_shadow(finish)
    }

    fn finish_fill(self, finish: Finish) -> Self {
        match finish {
            Finish::Card => self.bg(colors::SURFACE),
            Finish::Inset => self.bg(colors::INSET),
            Finish::Raised => self.bg(colors::RAISED),
            Finish::Light => self.bg(colors::LIGHT),
            Finish::Accent => self.bg(colors::ACCENT_FILL),
            Finish::Track => self.bg(colors::TRACK),
            Finish::Knob => self.bg(colors::KNOB),
            Finish::Pill => self.bg(colors::RAISED),
        }
    }

    fn finish_shadow(self, finish: Finish) -> Self {
        match finish {
            Finish::Card => self.adaptive_shadow(
                [drop(2.0, 8.0, rgb_a(0x000000, 0.04))],
                [drop(4.0, 12.0, rgb_a(0x000000, 0.28))],
            ),
            Finish::Inset => self.adaptive_shadow(
                [inner(1.0, 2.0, 0.0, rgb_a(0x000000, 0.04))],
                [inner(1.0, 2.0, 0.0, rgb_a(0x000000, 0.25))],
            ),
            Finish::Raised => self.adaptive_shadow(
                [drop(1.0, 3.0, rgb_a(0x000000, 0.05))],
                [drop(1.0, 3.0, rgb_a(0x000000, 0.20))],
            ),
            Finish::Light => self.adaptive_shadow(
                [drop(1.0, 4.0, rgb_a(0x000000, 0.10))],
                [drop(1.0, 4.0, rgb_a(0x000000, 0.35))],
            ),
            Finish::Accent => self.adaptive_shadow(
                [drop(2.0, 6.0, rgb_a(0x7caa14, 0.25))],
                [drop(2.0, 6.0, rgb_a(0xc8f26d, 0.20))],
            ),
            Finish::Track => self.adaptive_shadow(
                [inner(1.0, 2.0, 0.0, rgb_a(0x000000, 0.08))],
                [inner(1.0, 2.0, 0.0, rgb_a(0x000000, 0.30))],
            ),
            Finish::Knob => self.adaptive_shadow(
                [drop(1.0, 3.0, rgb_a(0x000000, 0.15))],
                [drop(1.0, 3.0, rgb_a(0x000000, 0.40))],
            ),
            Finish::Pill => self.adaptive_shadow(
                [drop(1.0, 2.0, rgb_a(0x000000, 0.05))],
                [drop(1.0, 2.0, rgb_a(0x000000, 0.20))],
            ),
        }
    }

    fn adaptive_shadow(
        self,
        light: impl IntoIterator<Item = BoxShadow>,
        dark: impl IntoIterator<Item = BoxShadow>,
    ) -> Self {
        let dark: Vec<BoxShadow> = dark.into_iter().collect();
        self.shadow(light)
            .dark(move |s: StyleRefinement| s.shadow(dark))
    }
}

impl<T: Styled + Sized> Finished for T {}

pub trait Clickable: Interactive + Styled + Sized {
    fn clickable(self) -> Self {
        self.focusable(true)
            .cursor_pointer()
            .transition_all()
            .duration(if reduce_motion() { 0.0 } else { 120.0 })
            .hover(|style| style.bg(colors::RAISED))
            .active(|style| style.bg(colors::BORDER).scale(0.97))
            .focus_visible(|style| style.border_1().border_color(colors::ACCENT))
    }
}

impl<T: Interactive + Styled + Sized> Clickable for T {}

pub trait Pressable: ComponentBuilder + Styled + Sized {
    fn pressable(self) -> Self {
        self.transition_transform()
            .duration(if reduce_motion() { 0.0 } else { 120.0 })
            .active(|style| style.scale(0.94))
    }
}

impl<T: ComponentBuilder + Styled + Sized> Pressable for T {}
