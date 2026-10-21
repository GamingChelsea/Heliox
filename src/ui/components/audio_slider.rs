use gpui_kit::gpui;
use gpui_kit::{
    App, Bounds, ElementId, InteractiveElement, MouseDownEvent, MouseMoveEvent, ParentElement,
    Pixels, RenderOnce, Styled, Window, base::ElementExt, component::ActiveTheme, div, px,
    relative,
};
use std::rc::Rc;

#[derive(Default)]
struct SliderInteraction {
    bounds: Bounds<Pixels>,
    dragging: bool,
}

#[derive(gpui_kit::prelude::IntoElement)]
pub struct AudioSlider {
    id: ElementId,
    value: f32,
    on_change: Option<Rc<dyn Fn(f32, &mut Window, &mut App)>>,
}
impl AudioSlider {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value: value.clamp(0.0, 1.0),
            on_change: None,
        }
    }

    pub fn on_change(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for AudioSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl gpui_kit::prelude::IntoElement {
        let value = self.value;
        let on_change = self.on_change;

        let interaction = window.use_keyed_state(
            ElementId::NamedChild(self.id.clone().into(), "interaction".into()),
            cx,
            |_, _| SliderInteraction::default(),
        );

        let update_value = {
            let interaction = interaction.clone();
            let on_change = on_change.clone();
            move |mouse_x: Pixels, window: &mut Window, cx: &mut App| {
                let bounds = interaction.read(cx).bounds;
                if bounds.size.width > px(0.0) {
                    let relative_x = (mouse_x - bounds.origin.x) / bounds.size.width;
                    let new_val = relative_x.clamp(0.0, 1.0);
                    if let Some(ref cb) = on_change {
                        cb(new_val, window, cx);
                    }
                }
            }
        };

        div()
            .id(self.id)
            .w_full()
            .h(px(16.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .on_prepaint({
                let interaction = interaction.clone();
                move |bounds, _window, cx| {
                    interaction.update(cx, |state, _cx| {
                        state.bounds = bounds;
                    });
                }
            })
            .on_mouse_down(gpui_kit::MouseButton::Left, {
                let interaction = interaction.clone();
                let update_value = update_value.clone();
                move |event: &MouseDownEvent, window, cx| {
                    interaction.update(cx, |state, _cx| {
                        state.dragging = true;
                    });
                    update_value(event.position.x, window, cx);
                }
            })
            .on_mouse_up(gpui_kit::MouseButton::Left, {
                let interaction = interaction.clone();
                move |_event, _window, cx| {
                    interaction.update(cx, |state, _cx| {
                        state.dragging = false;
                    });
                }
            })
            .on_mouse_up_out(gpui_kit::MouseButton::Left, {
                let interaction = interaction.clone();
                move |_event, _window, cx| {
                    interaction.update(cx, |state, _cx| {
                        state.dragging = false;
                    });
                }
            })
            .on_mouse_move({
                let interaction = interaction.clone();
                let update_value = update_value.clone();
                move |event: &MouseMoveEvent, window, cx| {
                    if interaction.read(cx).dragging {
                        update_value(event.position.x, window, cx);
                    }
                }
            })
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(3.0))
                    .bg(cx.theme().border)
                    .child(
                        div()
                            .h_full()
                            .w(relative(value))
                            .bg(cx.theme().primary)
                            .rounded(cx.theme().radius),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(-3.5))
                            .left(relative(value))
                            .ml(px(-5.0))
                            .size(px(10.0))
                            .rounded(cx.theme().radius)
                            .bg(cx.theme().foreground)
                            .shadow_sm(),
                    ),
            )
    }
}
