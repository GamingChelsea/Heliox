pub enum PaletteMode<'a> {
    Themes(&'a str),
    Navigation(&'a str),
    RepeatMode(&'a str),
    Actions,
}
impl<'a> PaletteMode<'a> {
    pub fn parse(query: &'a str) -> Self {
        let trimmed = query.trim_start();
        if let Some(rest) = trimmed.strip_prefix("ct ") {
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
