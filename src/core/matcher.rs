use nucleo_matcher::{Config, Matcher as NucleoMatcher, Utf32Str};

pub struct Matcher {
    config: Config,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            config: Config::DEFAULT,
        }
    }

    /// Performs fuzzy matching against `text` using `pattern`.
    /// Returns score if matched, or None if no match.
    pub fn fuzzy_score(&self, pattern: &str, text: &str) -> Option<u32> {
        if pattern.is_empty() {
            return Some(100);
        }
        let mut matcher = NucleoMatcher::new(self.config.clone());
        let mut pattern_buf = Vec::new();
        let mut text_buf = Vec::new();

        let pattern_utf32 = Utf32Str::new(pattern, &mut pattern_buf);
        let text_utf32 = Utf32Str::new(text, &mut text_buf);

        let score = matcher.fuzzy_match(text_utf32, pattern_utf32);
        score.map(|s| s as u32)
    }
}

impl Default for Matcher {
    fn default() -> Self {
        Self::new()
    }
}
