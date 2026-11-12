use crate::ui::{
    icons::{IconName, icon_elem},
    theme::colors,
};
use nsh_core::prelude::*;

pub fn cover_art(size_pt: f32) -> impl IntoElement {
    view()
        .size(px(size_pt))
        .rounded(px(size_pt * 0.16))
        .bg(colors::INSET)
        .border_1()
        .border_color(colors::BORDER)
        .items_center()
        .justify_center()
        .overflow_hidden()
        .child(icon_elem(IconName::Music, size_pt * 0.5, colors::MUTED))
}
