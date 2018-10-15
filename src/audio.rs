use ggez::{audio::Source, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Variants of sound effects which can be played.
pub enum SoundEffect {
    CreateRoute,
    ClickUIButton,
    PlaceShip,
    ProgressionStep,
}

/// Holds and manages playback of sound effects and music.
pub struct AudioHandler {
    sound_effects: Vec<(SoundEffect, Source)>,
    background_music: Source,
}

impl AudioHandler {
    /// Creates a new AudioHandler, loading and caching all audio resources.
    pub fn new(ctx: &mut Context) -> Self {
        let mut sound_effects = vec![
            (
                SoundEffect::ClickUIButton,
                Source::new(ctx, "/click.wav").expect("Failed to load click sound effect"),
            ),
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

        let mut background_music =
            Source::new(ctx, "/audionautix-metaphor.wav").expect("Failed to load main music");
        background_music.set_volume(0.2);
        background_music.set_repeat(true);

        // Lower volume for sound effects a bit.
        for (_, sound) in &mut sound_effects {
            sound.set_volume(0.3);
        }

        AudioHandler {
            sound_effects,
            background_music,
        }
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

    /// Starts playing the background music.
    pub fn start_music(&self) {
        self.background_music.play().expect("Failed to play music");
    }

    /// Stops playing the background music.
    pub fn stop_music(&self) {
        self.background_music.stop();
    }
}
