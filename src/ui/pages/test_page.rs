use gpui_kit::{component::label::Label, *};

#[derive(IntoElement)]
pub struct TestPage {
    message: String,
}
impl TestPage {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl RenderOnce for TestPage {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .size_full()
            .items_center()
            .justify_center()
            .content_center()
            .text_center()
            .child(Label::new(self.message))
    }
}
