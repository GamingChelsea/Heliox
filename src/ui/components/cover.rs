use gpui_kit::{
    App, ElementId, ObjectFit, Pixels, assets::IconName, base::h_flex, component::ActiveTheme,
    img, prelude::*, px,
};

pub fn cover_art(cx: &App) -> impl IntoElement {
    cover_art_sized("cover-art", px(56.), cx)
}

pub fn cover_art_sized(id: impl Into<ElementId>, size: Pixels, cx: &App) -> impl IntoElement {
    img("TODO.svg")
        .id(id)
        .size(size)
        .flex_shrink_0()
        .rounded(cx.theme().radius)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().muted)
        .overflow_hidden()
        .object_fit(ObjectFit::Contain)
        .with_loading(|| {
            h_flex()
                .size_full()
                .justify_center()
                .items_center()
                .child(IconName::Loader)
                .into_any_element()
        })
        .with_fallback(|| {
            h_flex()
                .size_full()
                .justify_center()
                .items_center()
                .child(IconName::Music)
                .into_any_element()
        })
}
