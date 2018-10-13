use super::*;

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

    /// Returns a ship to the shipyard.
    pub fn add_ship(&mut self, ship: Ship) {
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
