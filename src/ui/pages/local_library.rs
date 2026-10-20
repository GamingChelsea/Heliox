use crate::{model::CurrentTrackList, ui::components::track_list::TrackList};
use gpui_kit::{base::v_flex, component::label::Label, *};

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
                .child("No tracks found, or still loading...")
                .into_any_element();
        };

        v_flex()
            .size_full()
            .p_4()
            .gap_2()
            .child(
                v_flex()
                    .gap_1()
                    .child(Label::new("Local Library").text_3xl())
                    .child(Label::new("This is your local Library.").text_xs()),
            )
            .child(TrackList::new(tracks))
            .into_any_element()
    }
}
