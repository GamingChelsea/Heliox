use crate::providers::youtube::SearchFilter;

pub enum PaletteMode<'a> {
    Search(&'a str, SearchFilter),
    Themes(&'a str),
    Navigation(&'a str),
    RepeatMode(&'a str),
    Actions,
}
impl<'a> PaletteMode<'a> {
    pub fn parse(query: &'a str) -> Self {
        let trimmed = query.trim_start();
        if let Some(rest) = trimmed
            .strip_prefix("search:songs ")
            .or_else(|| trimmed.strip_prefix("search:song "))
            .or_else(|| trimmed.strip_prefix("songs: "))
            .or_else(|| trimmed.strip_prefix("song: "))
        {
            Self::Search(rest.trim(), SearchFilter::Songs)
        } else if let Some(rest) = trimmed
            .strip_prefix("search:videos ")
            .or_else(|| trimmed.strip_prefix("search:video "))
            .or_else(|| trimmed.strip_prefix("videos: "))
            .or_else(|| trimmed.strip_prefix("video: "))
        {
            Self::Search(rest.trim(), SearchFilter::Videos)
        } else if let Some(rest) = trimmed.strip_prefix("search ") {
            Self::Search(rest.trim(), SearchFilter::All)
        } else if let Some(rest) = trimmed.strip_prefix("ct ") {
            Self::Themes(rest.trim())
        } else if let Some(rest) = trimmed.strip_prefix("nav ") {
            Self::Navigation(rest.trim())
        } else if let Some(rest) = trimmed.strip_prefix("crm ") {
            Self::RepeatMode(rest.trim())
        } else {
            Self::Actions
        }
    }
}
