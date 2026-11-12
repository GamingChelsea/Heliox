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
        if let Some(rest) =
            command_argument(trimmed, &["search:songs", "search:song", "songs:", "song:"])
        {
            Self::Search(rest.trim(), SearchFilter::Songs)
        } else if let Some(rest) = command_argument(
            trimmed,
            &["search:videos", "search:video", "videos:", "video:"],
        ) {
            Self::Search(rest.trim(), SearchFilter::Videos)
        } else if let Some(rest) = command_argument(trimmed, &["search"]) {
            Self::Search(rest.trim(), SearchFilter::All)
        } else if let Some(rest) = command_argument(trimmed, &["ct"]) {
            Self::Themes(rest.trim())
        } else if let Some(rest) = command_argument(trimmed, &["nav"]) {
            Self::Navigation(rest.trim())
        } else if let Some(rest) = command_argument(trimmed, &["crm"]) {
            Self::RepeatMode(rest.trim())
        } else {
            Self::Actions
        }
    }
}

fn command_argument<'a>(query: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes.iter().find_map(|prefix| {
        query
            .strip_prefix(prefix)
            .filter(|rest| rest.chars().next().map_or(true, char::is_whitespace))
            .map(str::trim_start)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_prefixes_accept_the_prefix_without_a_trailing_space() {
        assert!(matches!(PaletteMode::parse("ct"), PaletteMode::Themes("")));
        assert!(matches!(
            PaletteMode::parse("nav"),
            PaletteMode::Navigation("")
        ));
        assert!(matches!(
            PaletteMode::parse("crm"),
            PaletteMode::RepeatMode("")
        ));
        assert!(matches!(
            PaletteMode::parse("search"),
            PaletteMode::Search("", SearchFilter::All)
        ));
    }

    #[test]
    fn command_prefixes_do_not_match_words_that_only_start_with_them() {
        assert!(matches!(
            PaletteMode::parse("navigation"),
            PaletteMode::Actions
        ));
    }
}
