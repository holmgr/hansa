use ggez::{
    graphics::{Color as ggezColor, DrawParam, Point2, Rect},
    nalgebra as na,
    timer::get_delta,
    Context,
};

use config::Config;
use draw::Drawable;
use geometry::Position;
use route::Waypoint;
use update::Updatable;
use world::World;

/// A ship which transports resources between ports along a route.
#[derive(Debug, Clone, PartialEq)]
pub struct Ship {
    position: Point2,
    current_waypoint: Waypoint,
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
            current_waypoint: path[0],
            position: Point2::from(Position::from(path[0])),
            path,
            reverse: false,
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
}

impl<'a> Updatable<'a> for Ship {
    type Data = Option<Vec<Waypoint>>;

    /// Updates the internal duration since last waypoint, moving to new
    /// waypoint if enough time has past.
    /// Note: The next path needs to be set if it is on the final waypoint,
    /// otherwise it can be omitted.
    fn update(&mut self, ctx: &Context, next_path: Option<Vec<Waypoint>>) {
        let current_waypoint = Point2::from(Position::from(self.current_waypoint));
        let next_waypoint = Point2::from(Position::from(self.next_waypoint().unwrap()));
        let distance_to_next = na::distance(&self.position, &next_waypoint);
        let delta = get_delta(ctx).as_secs() as f32 + get_delta(ctx).subsec_millis() as f32 / 1000.;
        let mut translation =
            na::normalize(&(next_waypoint - current_waypoint)) * Self::SPEED * delta;

        // Move to next waypoint.
        if na::norm(&translation) >= distance_to_next {
            self.current_waypoint = Waypoint::from(Position::from(next_waypoint));
            self.position = next_waypoint;
            let distance_remaining = na::norm(&translation) - distance_to_next;

            let next_waypoint =
                Point2::from(Position::from(match (self.next_waypoint(), next_path) {
                    (Some(w), _) => w,
                    (None, Some(ref nb)) if !nb.is_empty() => {
                        // TODO: Ugly code due to empty last path.
                        self.path = nb.clone();
                        self.next_waypoint().unwrap()
                    }
                    (None, _) => {
                        self.reverse = !self.reverse;
                        self.next_waypoint()
                            .expect("Could not find next waypoint after turning around")
                    }
                }));
            let current_waypoint = Point2::from(Position::from(self.current_waypoint));
            let next_waypoint = Point2::from(Position::from(next_waypoint));

            translation = na::normalize(&(next_waypoint - current_waypoint)) * distance_remaining;
        }
        self.position += translation;
    }
}

impl<'a> Drawable<'a> for Ship {
    type Data = (); // Mouse position

    fn draw(&self, _: &()) -> Vec<DrawParam> {
        let current_waypoint = Point2::from(Position::from(self.current_waypoint));
        let base_translation = na::normalize(&(self.position - current_waypoint));
        let rotation = (base_translation.y / base_translation.x).atan();

        // Add half cell to offset for rotation.
        let display_position =
            Point2::new(self.position.coords.x + 0.5, self.position.coords.y + 0.5);

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
