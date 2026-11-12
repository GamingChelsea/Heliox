use nsh_base::input;
use nsh_core::{element::IntoElement, geometry::Points, prelude::*};

use crate::ui::theme::colors;

pub fn text_input<F>(
    value: F,
    placeholder: impl Into<String>,
    on_change: impl Fn(&str) + 'static,
    on_submit: impl Fn(&str) + 'static,
) -> impl IntoElement
where
    F: Fn() -> String + Clone + 'static,
{
    let placeholder = placeholder.into();

    view()
        .relative()
        .flex_1()
        .h_full()
        .child(
            input()
                .absolute()
                .inset_0()
                .value(value.clone())
                .on_change(on_change)
                .on_submit(on_submit)
                .placeholder(placeholder.clone())
                .placeholder_color(|| color(colors::INPUT_MUTED))
                .caret_color(|| color(colors::ACCENT))
                .selection_color(|| color(colors::ACCENT))
                .text_size(px(15.0))
                .text_color(colors::INPUT_TEXT)
                .bg(colors::INPUT),
        )
        .when(cfg!(target_os = "windows"), move |field| {
            field.child(
                view()
                    .absolute()
                    .inset_0()
                    .flex_row()
                    .items_center()
                    .pointer_events(PointerEvents::None)
                    .accessibility_elements_hidden(true)
                    .child(visible_input_text(value, placeholder, px(15.0))),
            )
        })
}

pub fn visible_input_text<F>(
    value: F,
    placeholder: impl Into<String>,
    font_size: Points,
) -> impl IntoElement
where
    F: Fn() -> String + Clone + 'static,
{
    let placeholder = placeholder.into();
    let display_value = value.clone();

    text(move || {
        let current = display_value();
        if current.is_empty() {
            placeholder.clone()
        } else {
            current
        }
    })
    .number_of_lines(1)
    .truncate()
    .text_size(font_size)
    .text_color(move || {
        if value().is_empty() {
            colors::INPUT_MUTED
        } else {
            colors::INPUT_TEXT
        }
    })
}
