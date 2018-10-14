use ggez::{graphics::Font, Context};

/// Font cache which keeps track of the different font sizes.
pub struct FontCache {
    small: Font,
    medium: Font,
    large: Font,
}

impl FontCache {
    /// Creates a new FontCache.
    pub fn new(ctx: &mut Context) -> Self {
        FontCache {
            small: Font::new(ctx, "/RobotoMono-Regular.ttf", 32).expect("Failed to load font"),
            medium: Font::new(ctx, "/RobotoMono-Regular.ttf", 48).expect("Failed to load font"),
            large: Font::new(ctx, "/RobotoMono-Regular.ttf", 128).expect("Failed to load font"),
        }
    }

    /// Returns the small font.
    pub fn small(&self) -> &Font {
        &self.small
    }

    /// Returns the medium font.
    pub fn medium(&self) -> &Font {
        &self.medium
    }

    /// Returns the large font.
    pub fn large(&self) -> &Font {
        &self.large
    }
}
