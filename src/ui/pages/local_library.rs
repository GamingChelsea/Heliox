use crate::{model::CurrentTrackList, player::play_context, ui::components::track_list::TrackList};
use gpui_kit::{
    assets::IconName,
    base::{h_flex, v_flex},
    component::{button::Button, label::Label},
    *,
};

#[derive(IntoElement)]
pub struct LocalLibraryPage;
impl LocalLibraryPage {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for LocalLibraryPage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let tracks_opt = cx.global::<CurrentTrackList>().tracks.clone();

        let Some(tracks) = tracks_opt else {
            return div()
                .size_full()
                .items_center()
                .justify_center()
                .content_center()
                .text_center()
                .child("No tracks found, or still loading...")
                .into_any_element();
        };

        let tracks_for_play = tracks.clone();

        v_flex()
            .size_full()
            .p_4()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Label::new("Local Library").text_3xl())
                            .child(Label::new("This is your local Library.").text_xs()),
                    )
                    .child(
                        Button::new("play_button")
                            .icon(IconName::Play)
                            .child("Play playlist")
                            .on_click(move |_event, _window, cx| {
                                play_context(tracks_for_play.clone(), 0, cx);
                            }),
                    ),
            )
            .child(TrackList::new(tracks))
            .into_any_element()
    }
}
