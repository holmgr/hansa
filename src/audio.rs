use ggez::{audio::Source, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Variants of sound effects which can be played.
pub enum SoundEffect {
    CreateRoute,
    PlaceShip,
    ProgressionStep,
}

/// Holds and manages playback of sound effects and music.
pub struct AudioHandler {
    sound_effects: Vec<(SoundEffect, Source)>,
}

impl AudioHandler {
    /// Creates a new AudioHandler, loading and caching all audio resources.
    pub fn new(ctx: &mut Context) -> Self {
        let mut sound_effects = vec![
            (
                SoundEffect::CreateRoute,
                Source::new(ctx, "/click.wav").expect("Failed to load click sound effect"),
            ),
            (
                SoundEffect::PlaceShip,
                Source::new(ctx, "/click.wav").expect("Failed to load click sound effect"),
            ),
            (
                SoundEffect::ProgressionStep,
                Source::new(ctx, "/blip.wav").expect("Failed to load blip sound effect"),
            ),
        ];

        // Lower volume for sound effects a bit.
        for (_, sound) in &mut sound_effects {
            sound.set_volume(0.3);
        }

        AudioHandler { sound_effects }
    }

    /// Plays the sound effect given once.
    pub fn play(&self, effect: SoundEffect) {
        let (_, sound) = self
            .sound_effects
            .iter()
            .find(|(e, _)| *e == effect)
            .expect("Did not find associated sound effect");
        sound.play().expect("Failed to play sound");
    }
}
