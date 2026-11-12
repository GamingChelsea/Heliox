use nsh_base::{SliderValue, slider};
use nsh_core::prelude::*;
use std::rc::Rc;

pub fn audio_slider(
    value: impl IntoReactive<f32>,
    on_change: impl Fn(f32) + 'static,
) -> impl IntoElement {
    audio_slider_with_track(value, 0.001, 0.02, false, on_change)
}

pub fn seek_slider(
    value: impl IntoReactive<f32>,
    on_change: impl Fn(f32) + 'static,
) -> impl IntoElement {
    audio_slider_with_track(value, 0.001, 0.02, true, on_change)
}

pub fn audio_slider_with_step(
    value: impl IntoReactive<f32>,
    step: f64,
    on_change: impl Fn(f32) + 'static,
) -> impl IntoElement {
    audio_slider_with_track(value, step, step, false, on_change)
}

fn audio_slider_with_track(
    value: impl IntoReactive<f32>,
    slider_step: f64,
    wheel_step: f64,
    thin_track: bool,
    on_change: impl Fn(f32) + 'static,
) -> impl IntoElement {
    let value = value.into_reactive();
    let wheel_value = value.clone();
    let on_change = Rc::new(on_change);
    let on_wheel_change = on_change.clone();
    let indicator_class = if thin_track {
        "h-1 rounded-full bg-muted"
    } else {
        "h-1.5 rounded-full bg-muted"
    };
    view()
        .h(px(24.0))
        .flex_1()
        .on_wheel(move |event: &WheelEvent| {
            let (_, delta_y) = event.delta.xy();
            if delta_y != 0.0 {
                let direction = if delta_y < 0.0 { 1.0 } else { -1.0 };
                let next = (wheel_value.get() as f64 + direction * wheel_step).clamp(0.0, 1.0);
                on_wheel_change(next as f32);
            }
        })
        .child(
            slider()
                .value(value.map(|v| Some(SliderValue::Single(v as f64))))
                .min(0.0)
                .max(1.0)
                .step(slider_step)
                .track_class("h-6 justify-center")
                .indicator_class(indicator_class)
                .range_class("rounded-full bg-primary")
                .thumb_class("size-4 rounded-full bg-background")
                .on_change(move |&v: &SliderValue| on_change(v.start() as f32))
                .h(px(24.0))
                .flex_1(),
        )
}
