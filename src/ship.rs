use ggez::{
    graphics::{Color as ggezColor, DrawParam, Point2, Rect},
    timer::get_delta,
    Context,
};
use std::time::Duration;

use config::Config;
use draw::Drawable;
use geometry::Position;
use route::Waypoint;
use update::Updatable;
use world::World;

/// A ship which transports resources between ports along a route.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Ship {
    /// Duration since arriving at current position.
    duration: Duration,
    position: Waypoint,
    /// Current path.
    path: Vec<Waypoint>,
    /// If we are on the return trip or not.
    reverse: bool,
}

impl Ship {
    const SPEED: f32 = 5.;

    /// Creates a new ship.
    pub fn new(path: Vec<Waypoint>) -> Self {
        Ship {
            duration: Duration::new(0, 0),
            position: path[0],
            path,
            reverse: false,
        }
    }

    /// Returns the ship's current position.
    pub fn position(&self) -> Waypoint {
        self.position
    }

    /// Returns whether the ship currently is on its return trip.
    pub fn reverse(&self) -> bool {
        self.reverse
    }

    /// Returns the next waypoint on the ship's current route.
    /// Will return the 'previous' waypoint if based on reverse state.
    pub fn next_waypoint(&self) -> Option<Waypoint> {
        let current_position = self
            .path
            .iter()
            .position(|w| *w == self.position)
            .expect("Current position not on path");
        if self.reverse {
            self.path.get(current_position - 1).cloned()
        } else {
            self.path.get(current_position + 1).cloned()
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
        let current_position = Position::from(self.position);

        if let Some(next_path) = next_path {
            // TODO: Ugly code due to empty last path.
            if !next_path.is_empty() {
                self.path = next_path;
            }
        }
        let next_position = Position::from(match self.next_waypoint() {
            Some(w) => w,
            None => {
                self.reverse = !self.reverse;
                if self.next_waypoint().is_none() {
                    println!("{:#?}", self.path.iter().rev());
                }
                self.next_waypoint()
                    .expect("Could not find next position after turning around")
            }
        });

        let distance_to_next = current_position.distance(next_position);
        let delta_since_move = get_delta(ctx) + self.duration;

        // Move to next ewaypoint.
        if (delta_since_move.as_secs() as u32 * 1000 + delta_since_move.subsec_millis()) as f32
            * Self::SPEED
            / 1000.
            >= distance_to_next
        {
            self.position = Waypoint::from(next_position);
            self.duration = Duration::new(0, 0); // Does not consider if we over reach!!!
        } else {
            self.duration = delta_since_move;
        }
    }
}

impl<'a> Drawable<'a> for Ship {
    type Data = (); // Mouse position

    fn draw(&self, _: &()) -> Vec<DrawParam> {
        let current_position = Position::from(self.position);
        let (mut interpolated_position, rotation) = match self.next_waypoint() {
            Some(p) => {
                let next_position = Position::from(p);
                let base_translation = next_position - current_position;
                let magnitute = base_translation.distance_origo();
                let distance_traveled = (self.duration.as_secs() as u32 * 1000
                    + self.duration.subsec_millis()) as f32
                    * Self::SPEED
                    / 1000.;
                //println!("Distance traveled: {}, magnitude: {}", distance_traveled, magnitute);
                let translate_factor = distance_traveled / magnitute;
                //println!("Translate factor: {}", translate_factor);

                (
                    Point2::new(
                        current_position.x as f32 + (base_translation.x as f32 * translate_factor),
                        current_position.y as f32 + (base_translation.y as f32 * translate_factor),
                    ),
                    base_translation.direction(),
                )
            }
            //println!("Drawing ship on position: {}", interpolated_position);
            None => {
                println!("No next position when drawing, fallback to same position");
                (Point2::from(current_position), 0.)
            }
        };

        // Add half cell to offset for rotation.
        interpolated_position.coords.x += 0.5;
        interpolated_position.coords.y += 0.5;

        vec![DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: interpolated_position,
            rotation,
            offset: Point2::new(0.5, 0.5),
            color: Some(ggezColor::from_rgb(69, 55, 52)),
            ..Default::default()
        }]
    }
}

/// Holds all unplaced 'ships' and manages of drawing UI element for ship selection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shipyard {
    ships: usize,
}

impl Shipyard {
    /// Create a new shipyard.
    pub fn new() -> Self {
        Shipyard {
            ships: 3, // TODO: Make this scale with progress.
        }
    }

    /// Returns the number of available ships.
    pub fn available(&self) -> usize {
        self.ships
    }

    /// Return if there are any ships available.
    pub fn is_available(&self) -> bool {
        self.ships > 0
    }

    /// Returns a shipbuilder if not capped on ships.
    pub fn build(&mut self) -> Option<ShipBuilder> {
        if self.is_available() {
            self.ships -= 1;
            Some(ShipBuilder::new())
        } else {
            None
        }
    }

    /// Returns a builder to the shipyard.
    pub fn add_builder(&mut self, builder: ShipBuilder) {
        self.ships += 1;
    }
}

impl<'a> Drawable<'a> for Shipyard {
    type Data = Config;

    fn draw(&self, data: &Config) -> Vec<DrawParam> {
        let x_offset = (data.grid_width / 2) as f32;
        let y_offset = data.grid_height as f32 + 3.;
        vec![DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(x_offset, y_offset),
            color: Some(ggezColor::from_rgb(69, 55, 52)),
            offset: Point2::new(0.5, 0.5),
            ..Default::default()
        }]
    }
}

/// Manages the placement of ships on routes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ShipBuilder;

impl ShipBuilder {
    /// Create a new ship builder.
    pub fn new() -> Self {
        ShipBuilder {}
    }

    /// Attempts to place a ship at the given position, consuming the ship builder.
    /// Returns the builder if it failed.
    pub fn try_place(self, position: Position, world: &mut World) -> Option<ShipBuilder> {
        // TODO: Implement placement on the given possition if it exists a
        // valid path there.

        let waypoint = Waypoint::from(position);
        if let Some((_, route)) = world.routes_mut().find(|(_, r)| r.contains(waypoint)) {
            println!("Placed ship on route!");
            let initial_path = route.initial_path();
            route.add_ship(Ship::new(initial_path));
            None
        } else {
            Some(self)
        }
    }
}

impl<'a> Drawable<'a> for ShipBuilder {
    type Data = Point2; // Mouse position

    fn draw(&self, mouse: &Point2) -> Vec<DrawParam> {
        vec![DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(mouse.coords.x, mouse.coords.y),
            offset: Point2::new(0.5, 0.5),
            color: Some(ggezColor::from_rgb(69, 55, 52)),
            ..Default::default()
        }]
    }
}
