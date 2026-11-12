use nsh_core::prelude::*;

use crate::{
    router::go_back,
    ui::{
        icons::{IconName, icon_elem},
        theme::{Clickable, colors},
    },
};

pub fn test_page(message: String) -> impl IntoElement {
    view()
        .flex_1()
        .items_center()
        .justify_center()
        .p(px(24.0))
        .gap(px(16.0))
        .child(
            text("Test Page")
                .text_size(px(24.0))
                .font_weight(FontWeight::BOLD)
                .text_color(colors::TEXT),
        )
        .child(text(message).text_size(px(16.0)).text_color(colors::MUTED))
        .child(
            view()
                .flex_row()
                .items_center()
                .h(px(36.0))
                .px(px(16.0))
                .gap(px(8.0))
                .rounded(px(8.0))
                .bg(colors::RAISED)
                .border_1()
                .border_color(colors::BORDER)
                .clickable()
                .label("Go Back")
                .on_press(go_back)
                .child(icon_elem(IconName::ChevronLeft, 16.0, colors::TEXT))
                .child(
                    text("Go Back")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::TEXT),
                ),
        )
}
