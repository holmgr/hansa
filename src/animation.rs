use ggez::{
    graphics::{Color as ggezColor, DrawParam},
    timer::{duration_to_f64, get_delta},
    Context,
};
use std::{f32::consts::PI, time::Duration};

use color::Color;
use update::Updatable;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationType {
    PulseScale {
        amplitude: f32,
        rate: f32,
    },
    ColorDrain {
        from: Option<Color>,
        to: Option<Color>,
    },
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
                AnimationType::ColorDrain { from, to } => {
                    // If over half way.
                    let animation_fraction = duration_to_f64(self.time_elapsed)
                        / duration_to_f64(self.animation_duration);
                    println!(
                        "Elapsed: {:?}, Total: {:?}, Animation fraction: {}",
                        self.time_elapsed, self.animation_duration, animation_fraction
                    );
                    let black = (69, 55, 52);

                    let (from_r, from_g, from_b) = match from {
                        Some(c) => c.rgb(),
                        None => black,
                    };
                    let (to_r, to_g, to_b) = match to {
                        Some(c) => c.rgb(),
                        None => black,
                    };
                    let from_amount = 1. - animation_fraction;
                    let to_amount = 1. - from_amount;
                    param.color = Some(ggezColor::from_rgb(
                        (f64::from(from_r) * from_amount + f64::from(to_r) * to_amount) as u8,
                        (f64::from(from_g) * from_amount + f64::from(to_g) * to_amount) as u8,
                        (f64::from(from_b) * from_amount + f64::from(to_b) * to_amount) as u8,
                    ));
                }
            }
        }
        drawparams
    }
}

impl<'a> Updatable<'a> for Animation {
    type Data = ();

    fn update(&'a mut self, ctx: &'a Context, _data: ()) {
        self.time_elapsed += get_delta(ctx)
    }
}
