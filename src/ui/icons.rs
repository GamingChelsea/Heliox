//! Canvas-rendered vector icons for Heliox (in pure Rust with NSH).

use std::f32::consts::TAU;

use nsh_core::{
    canvas::{LineCap, LineJoin},
    element::{JsObject, PropEnum},
    prelude::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum IconName {
    #[default]
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Repeat,
    Repeat1,
    RepeatOff,
    Shuffle,
    House,
    Search,
    Library,
    Settings,
    TestTube,
    PanelLeftClose,
    PanelLeftOpen,
    Music,
    ListMusic,
    Volume,
    VolumeX,
    Folder,
    Check,
    Plus,
    Cross,
    ChevronRight,
    ChevronLeft,
    Spark,
}

nsh_core::static_reactive!(IconName: IconName);

fn line(p: Painter, points: &[f32]) -> Painter {
    let mut p = p.begin_path().move_to(points[0], points[1]);
    for xy in points[2..].chunks(2) {
        p = p.line_to(xy[0], xy[1]);
    }
    p.stroke()
}

fn rect(p: Painter, x: f32, y: f32, w: f32, h: f32, r: f32) -> Painter {
    p.begin_path()
        .round_rect(
            nsh_core::geometry::Bounds::new(Point::new(px(x), px(y)), Size::new(px(w), px(h))),
            px(r),
        )
        .stroke()
}

fn circle(p: Painter, x: f32, y: f32, r: f32, fill: bool) -> Painter {
    let p = p.begin_path().arc(x, y, r, 0.0, TAU);
    if fill { p.fill() } else { p.stroke() }
}

impl IconName {
    pub fn draw(self, c: Painter) -> Painter {
        match self {
            IconName::Play => c
                .begin_path()
                .move_to(6.5, 4.5)
                .line_to(15.5, 10.0)
                .line_to(6.5, 15.5)
                .close_path()
                .fill(),
            IconName::Pause => c
                .fill_rect(5.5, 4.5, 3.0, 11.0)
                .fill_rect(11.5, 4.5, 3.0, 11.0),
            IconName::SkipBack => {
                let c = c.fill_rect(4.0, 4.5, 2.5, 11.0);
                c.begin_path()
                    .move_to(15.5, 4.5)
                    .line_to(7.5, 10.0)
                    .line_to(15.5, 15.5)
                    .close_path()
                    .fill()
            }
            IconName::SkipForward => {
                let c = c.fill_rect(13.5, 4.5, 2.5, 11.0);
                c.begin_path()
                    .move_to(4.5, 4.5)
                    .line_to(12.5, 10.0)
                    .line_to(4.5, 15.5)
                    .close_path()
                    .fill()
            }
            IconName::Repeat => {
                let c = line(c, &[14.0, 5.0, 17.0, 8.0, 14.0, 11.0]);
                let c = line(c, &[4.0, 10.0, 4.0, 8.0, 16.5, 8.0]);
                let c = line(c, &[6.0, 15.0, 3.0, 12.0, 6.0, 9.0]);
                line(c, &[16.0, 10.0, 16.0, 12.0, 3.5, 12.0])
            }
            IconName::Repeat1 => {
                let c = IconName::Repeat.draw(c);
                line(c, &[10.0, 8.5, 10.0, 11.5])
            }
            IconName::RepeatOff => {
                let c = IconName::Repeat.draw(c);
                line(c, &[3.0, 3.0, 17.0, 17.0])
            }
            IconName::Shuffle => {
                let c = line(c, &[13.0, 4.0, 17.0, 7.0, 13.0, 10.0]);
                let c = line(c, &[3.0, 13.0, 7.0, 13.0, 13.0, 7.0, 16.5, 7.0]);
                let c = line(c, &[13.0, 16.0, 17.0, 13.0, 13.0, 10.0]);
                let c = line(c, &[3.0, 7.0, 7.0, 7.0, 9.5, 9.5]);
                line(c, &[11.5, 11.5, 13.0, 13.0, 16.5, 13.0])
            }
            IconName::House => {
                let c = line(c, &[3.0, 10.0, 10.0, 3.5, 17.0, 10.0]);
                line(c, &[5.0, 9.0, 5.0, 16.5, 15.0, 16.5, 15.0, 9.0])
            }
            IconName::Search => {
                let c = circle(c, 8.5, 8.5, 5.0, false);
                line(c, &[12.3, 12.3, 16.5, 16.5])
            }
            IconName::Library => {
                let c = line(c, &[4.0, 4.0, 4.0, 16.0]);
                let c = line(c, &[8.0, 4.0, 8.0, 16.0]);
                let c = line(c, &[12.0, 5.0, 15.0, 16.0]);
                rect(c, 2.5, 3.0, 15.0, 14.0, 2.0)
            }
            IconName::Settings => {
                let c = circle(c, 10.0, 10.0, 3.5, false);
                [0.0, 45.0, 90.0, 135.0].into_iter().fold(c, |c, _deg| {
                    line(c, &[10.0, 2.5, 10.0, 4.5, 10.0, 15.5, 10.0, 17.5])
                })
            }
            IconName::TestTube => {
                let c = line(c, &[7.0, 3.0, 13.0, 3.0]);
                let c = line(c, &[8.5, 3.0, 8.5, 12.0]);
                let c = line(c, &[11.5, 3.0, 11.5, 12.0]);
                circle(c, 10.0, 14.5, 3.0, false)
            }
            IconName::PanelLeftClose => {
                let c = rect(c, 3.0, 3.5, 14.0, 13.0, 2.5);
                let c = line(c, &[8.0, 3.5, 8.0, 16.5]);
                line(c, &[13.5, 8.0, 11.5, 10.0, 13.5, 12.0])
            }
            IconName::PanelLeftOpen => {
                let c = rect(c, 3.0, 3.5, 14.0, 13.0, 2.5);
                let c = line(c, &[8.0, 3.5, 8.0, 16.5]);
                line(c, &[11.5, 8.0, 13.5, 10.0, 11.5, 12.0])
            }
            IconName::Music => {
                let c = circle(c, 6.5, 14.0, 2.5, true);
                let c = line(c, &[9.0, 14.0, 9.0, 4.5, 15.0, 3.5, 15.0, 11.5]);
                circle(c, 12.5, 12.5, 2.5, true)
            }
            IconName::ListMusic => {
                let c = line(c, &[3.5, 6.0, 11.5, 6.0]);
                let c = line(c, &[3.5, 10.0, 10.0, 10.0]);
                let c = line(c, &[3.5, 14.0, 9.0, 14.0]);
                let c = circle(c, 13.5, 14.5, 2.0, true);
                line(c, &[15.5, 14.5, 15.5, 8.0, 17.5, 7.5])
            }
            IconName::Volume => {
                let c = line(
                    c,
                    &[
                        4.0, 8.0, 7.0, 8.0, 11.0, 5.0, 11.0, 15.0, 7.0, 12.0, 4.0, 12.0, 4.0, 8.0,
                    ],
                );
                let c = line(c, &[13.5, 7.5, 14.5, 8.5, 14.5, 11.5, 13.5, 12.5]);
                line(c, &[15.5, 5.5, 17.5, 7.5, 17.5, 12.5, 15.5, 14.5])
            }
            IconName::VolumeX => {
                let c = line(
                    c,
                    &[
                        4.0, 8.0, 7.0, 8.0, 11.0, 5.0, 11.0, 15.0, 7.0, 12.0, 4.0, 12.0, 4.0, 8.0,
                    ],
                );
                let c = line(c, &[13.5, 8.0, 17.5, 12.0]);
                line(c, &[17.5, 8.0, 13.5, 12.0])
            }
            IconName::Folder => {
                let c = line(
                    c,
                    &[
                        3.0, 6.0, 3.0, 15.5, 17.0, 15.5, 17.0, 8.0, 10.0, 8.0, 8.0, 6.0, 3.0, 6.0,
                    ],
                );
                line(c, &[3.0, 9.0, 17.0, 9.0])
            }
            IconName::Check => line(c, &[4.5, 10.5, 8.5, 14.0, 15.5, 6.0]),
            IconName::Plus => {
                let c = line(c, &[10.0, 4.0, 10.0, 16.0]);
                line(c, &[4.0, 10.0, 16.0, 10.0])
            }
            IconName::Cross => {
                let c = line(c, &[5.0, 5.0, 15.0, 15.0]);
                line(c, &[15.0, 5.0, 5.0, 15.0])
            }
            IconName::ChevronRight => line(c, &[7.5, 5.0, 12.5, 10.0, 7.5, 15.0]),
            IconName::ChevronLeft => line(c, &[12.5, 5.0, 7.5, 10.0, 12.5, 15.0]),
            IconName::Spark => line(
                c,
                &[
                    10.0, 3.0, 11.5, 8.5, 17.0, 10.0, 11.5, 11.5, 10.0, 17.0, 8.5, 11.5, 3.0, 10.0,
                    8.5, 8.5, 10.0, 3.0,
                ],
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, JsObject)]
pub struct Tint {
    pub light: Hsla,
    pub dark: Hsla,
}

impl From<AdaptiveColor> for Tint {
    fn from(c: AdaptiveColor) -> Self {
        Self {
            light: c.light,
            dark: c.dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Props)]
pub struct IconProps {
    pub name: Prop<IconName>,
    #[prop(default = 20.0)]
    pub size: Prop<f32>,
    pub tint: Prop<Tint>,
    pub label: Prop<Option<SharedString>>,
}

#[component]
pub fn Icon(props: IconProps) -> impl IntoElement {
    view()
        .size(move || px(props.size.get()))
        .pointer_events(PointerEvents::None)
        .child(
            canvas(move || {
                let tint = props.tint.get();
                let tint = color(AdaptiveColor::new(tint.light, tint.dark));
                let scale = props.size.get() / 20.0;
                let painter = Painter::new()
                    .scale(scale, scale)
                    .stroke_color(tint)
                    .fill_color(tint)
                    .line_width(1.6)
                    .line_cap(LineCap::Round)
                    .line_join(LineJoin::Round);
                props.name.get().draw(painter)
            })
            .size_full(),
        )
}

pub fn icon_elem(name: IconName, size_pt: f32, tint_color: AdaptiveColor) -> impl IntoElement {
    icon().name(name).size(size_pt).tint(Tint::from(tint_color))
}
