use gpui_kit::{component::label::Label, *};

#[derive(IntoElement)]
pub struct HomePage;
impl HomePage {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for HomePage {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        Label::new("Home")
    }
}
