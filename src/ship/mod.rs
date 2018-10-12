use ggez::{
    graphics::{Color as ggezColor, DrawParam, Point2, Rect},
    nalgebra as na,
    timer::get_delta,
    Context,
};
use std::time::Duration;

use animation::Animation;
use color::Color;
use config::Config;
use draw::Drawable;
use geometry::Position;
use route::Waypoint;
use update::Updatable;
use world::World;

mod shipbuilder;
mod shipyard;

pub use self::shipbuilder::ShipBuilder;
pub use self::shipyard::Shipyard;

/// A ship which transports resources between ports along a route.
#[derive(Debug, Clone, PartialEq)]
pub struct Ship {
    docked: Duration,
    position: Point2,
    current_waypoint: Waypoint,
    /// Current path.
    path: Vec<Waypoint>,
    /// If we are on the return trip or not.
    reverse: bool,
    /// Color currently being carried from the given port.
    cargo: Option<(Waypoint, Color)>,
    animation: Option<Animation>,
}

impl Ship {
    const SPEED: f32 = 5.;

    /// Creates a new ship.
    pub fn new(position: Waypoint, path: Vec<Waypoint>) -> Self {
        Ship {
            docked: Duration::from_millis(0),
            current_waypoint: position,
            position: Point2::from(Position::from(position)),
            path,
            reverse: false,
            cargo: None,
            animation: None,
        }
    }

    /// Returns the ship's current position.
    pub fn position(&self) -> Waypoint {
        self.current_waypoint
    }

    /// Returns whether the ship currently is on its return trip.
    pub fn reverse(&self) -> bool {
        self.reverse
    }

    pub fn animation_mut(&mut self) -> &mut Option<Animation> {
        &mut self.animation
    }

    /// Returns the next waypoint on the ship's current route.
    /// Will return the 'previous' waypoint if based on reverse state.
    pub fn next_waypoint(&self) -> Option<Waypoint> {
        let current_position = self
            .path
            .iter()
            .position(|w| *w == self.current_waypoint)
            .expect("Current position not on path");
        if self.reverse {
            self.path.get(current_position - 1).cloned()
        } else {
            self.path.get(current_position + 1).cloned()
        }
    }

    /// Returns whether the ships is currently docked.
    pub fn is_docked(&self) -> bool {
        self.docked.subsec_millis() > 0
    }

    /// Returns the current cargo if any.
    pub fn cargo(&self) -> Option<Color> {
        self.cargo.map(|(_, c)| c)
    }

    /// Attempts to unload the current cargo if any.
    pub fn try_unload(&mut self) -> Option<Color> {
        let tmp = match self.cargo {
            Some((w, c)) if w != self.current_waypoint => Some(c),
            _ => None,
        };
        if tmp.is_some() {
            self.cargo = None;
        }
        tmp
    }

    /// Attempts to load the given color as cargo, doing nothing if already loaded.
    pub fn try_load(&mut self, color: Color) {
        self.cargo = self.cargo.or_else(|| Some((self.current_waypoint, color)));
    }

    /// Returns true if the next waypoint is the final waypoint on the path.
    pub fn is_arriving(&self) -> bool {
        let current_position = self
            .path
            .iter()
            .position(|w| *w == self.current_waypoint)
            .expect("Current position not on path");
        if self.reverse {
            self.path.get(current_position - 2).is_none()
        } else {
            self.path.get(current_position + 2).is_none()
        }
    }

    /// Returns true if the current waypoint is the first waypoint on the path.
    pub fn is_leaving(&self) -> bool {
        let current_position = self
            .path
            .iter()
            .position(|w| *w == self.current_waypoint)
            .expect("Current position not on path");
        if self.reverse {
            current_position == self.path.len() - 1
        } else {
            current_position == 0
        }
    }
}

impl<'a> Updatable<'a> for Ship {
    type Data = Option<Vec<Waypoint>>;

    /// Updates the internal duration since last waypoint, moving to new
    /// waypoint if enough time has past.
    /// Note: The next path needs to be set if it is on the final waypoint,
    /// otherwise it can be omitted.
    fn update(&mut self, ctx: &Context, next_path: Option<Vec<Waypoint>>) {
        if !self.is_docked() {
            let current_waypoint = Point2::from(Position::from(self.current_waypoint));
            let next_waypoint = Point2::from(Position::from(self.next_waypoint().unwrap()));
            let distance_to_next = na::distance(&self.position, &next_waypoint);
            let delta =
                get_delta(ctx).as_secs() as f32 + get_delta(ctx).subsec_millis() as f32 / 1000.;

            let mut translation = na::normalize(&(next_waypoint - current_waypoint))
                * Self::SPEED
                * delta
                * match (self.is_arriving(), self.is_leaving()) {
                    (true, _) => distance_to_next.powf(1.3).max(0.2),
                    (_, true) => (1. - distance_to_next).powf(1.3).max(0.2),
                    (_, _) => 1.,
                };

            // Move to next waypoint.
            if na::norm(&translation) >= distance_to_next {
                self.current_waypoint = Waypoint::from(Position::from(next_waypoint));
                self.position = next_waypoint;
                let distance_remaining = na::norm(&translation) - distance_to_next;

                if let Some(w) = self.next_waypoint() {
                    let current_waypoint = next_waypoint;
                    let next_waypoint = Point2::from(Position::from(w));

                    translation =
                        na::normalize(&(next_waypoint - current_waypoint)) * distance_remaining;
                } else {
                    match next_path {
                        Some(ref nb) if !nb.is_empty() => {
                            // TODO: Ugly code due to empty last path.
                            self.path = nb.clone();
                            self.next_waypoint().unwrap()
                        }
                        _ => {
                            self.reverse = !self.reverse;
                            self.next_waypoint()
                                .expect("Could not find next waypoint after turning around")
                        }
                    };
                    // Set dock timer.
                    self.docked = Duration::from_millis(999);
                }
            }
            self.position += translation;
        } else {
            // Reduce time remaining, setting to zero if underflow etc.
            self.docked = self
                .docked
                .checked_sub(get_delta(ctx))
                .unwrap_or(Duration::from_millis(0));
        }
    }
}

impl<'a> Drawable<'a> for Ship {
    type Data = (); // Mouse position

    fn animation(&self) -> Option<Animation> {
        self.animation
    }

    fn draw(&self, _: &()) -> Vec<DrawParam> {
        let current_waypoint = Point2::from(Position::from(self.current_waypoint));
        let base_translation = na::normalize(&(self.position - current_waypoint));
        let rotation = (base_translation.y / base_translation.x).atan();

        // Add half cell to offset for rotation.
        let display_position =
            Point2::new(self.position.coords.x + 0.5, self.position.coords.y + 0.5);

        let (r, g, b) = match self.cargo {
            Some((_, color)) => color.rgb(),
            None => (69, 55, 52),
        };

        vec![DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: display_position,
            rotation,
            offset: Point2::new(0.5, 0.5),
            color: Some(ggezColor::from_rgb(r, g, b)),
            ..Default::default()
        }]
    }
}
