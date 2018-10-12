use ggez::{
    graphics::DrawParam,
    timer::{duration_to_f64, get_delta},
    Context,
};
use std::{f32::consts::PI, time::Duration};

use update::Updatable;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationType {
    PulseScale { amplitude: f32, rate: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animation {
    time_elapsed: Duration,
    animation_duration: Duration,
    animation_type: AnimationType,
}

impl Eq for Animation {}

/// An animation that can be applied to drawable types.
impl Animation {
    /// Creates a new animation of the given type.
    pub fn new(animation_duration: Duration, animation_type: AnimationType) -> Self {
        Animation {
            time_elapsed: Duration::default(),
            animation_duration,
            animation_type,
        }
    }

    /// Returns the total animation duration.
    pub fn animation_duration(&self) -> Duration {
        self.animation_duration
    }

    /// Return the amount of time passed since animation start.
    pub fn duration_elapsed(&self) -> Duration {
        self.time_elapsed
    }

    /// Return a mutable reference to the amount of time passed since animation start.
    pub fn duration_elapsed_mut(&mut self) -> &mut Duration {
        &mut self.time_elapsed
    }

    /// Returns whether the animation has finished.
    pub fn has_finished(&self) -> bool {
        self.duration_elapsed() >= self.animation_duration()
    }

    /// Animates the given drawparams.
    pub fn animate(&self, mut drawparams: Vec<DrawParam>) -> Vec<DrawParam> {
        for param in &mut drawparams {
            match self.animation_type {
                AnimationType::PulseScale { amplitude, rate } => {
                    param.scale *= amplitude
                        * (2. * PI * duration_to_f64(self.time_elapsed) as f32 / rate).sin()
                        + 1.;
                }
            }
        }
        drawparams
    }
}

impl<'a> Updatable<'a> for Animation {
    type Data = ();

    fn update(&'a mut self, ctx: &'a Context, _data: ()) {
        let duration_elapsed = self.duration_elapsed_mut();
        *duration_elapsed += get_delta(ctx)
    }
}
