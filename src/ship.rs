use ggez::graphics::{Color as ggezColor, DrawParam, Point2, Rect};

use config::Config;
use draw::Drawable;
use world::World;
use Position;

/// A ship which transports resources between ports along a route.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ship {}

impl Ship {
    pub fn new() -> Self {
        // TODO: Implement this
        Ship {}
    }
}

/// Holds all unplaced ships and manages of drawing UI element for ship selection.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Shipyard {
    ships: Vec<Ship>,
}

impl Shipyard {
    /// Create a new shipyard.
    pub fn new() -> Self {
        Shipyard {
            ships: vec![Ship::new()],
        }
    }

    /// Returns an available ship, if any is ready.
    pub fn get_available_ship(&mut self) -> Option<Ship> {
        self.ships.pop()
    }

    /// Returns the number of available ships.
    pub fn available(&self) -> usize {
        self.ships.len()
    }

    /// Return if there are any ships available.
    pub fn is_available(&self) -> bool {
        !self.ships.is_empty()
    }

    /// Adds a new ship to the shipyard.
    pub fn add_ship(&mut self, ship: Ship) {
        self.ships.push(ship);
    }
}

impl<'a> Drawable<'a> for Shipyard {
    type Data = Config;

    fn draw(&self, data: &Config) -> DrawParam {
        let x_offset = (data.grid_width / 2) as f32;
        let y_offset = data.grid_height as f32 + 3.;
        DrawParam {
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
        }
    }
}

/// Manages the placement of ships on routes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ShipBuilder {
    ship: Ship,
}

impl ShipBuilder {
    /// Create a new ship builder.
    pub fn new(ship: Ship) -> Self {
        ShipBuilder { ship }
    }

    /// Attempts to place a ship at the given position, consuming the ship builder
    /// if it fails.
    pub fn place(self, position: Position, world: &World) -> Option<Ship> {
        // TODO: Implement placement on the given possition if it exists a
        // valid path there.
        None
    }

    /// Cancels the ship placement, consuming the builder and returning the ship.
    pub fn cancel(self) -> Ship {
        self.ship
    }
}

impl<'a> Drawable<'a> for ShipBuilder {
    type Data = Point2; // Mouse position

    fn draw(&self, mouse: &Point2) -> DrawParam {
        DrawParam {
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
        }
    }
}
