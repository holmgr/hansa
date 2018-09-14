use std::{collections::HashMap, iter::FromIterator, iter::Iterator};
use tile::Tile;

use Position;

/// Holds all information on the game world.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct World {
    map: HashMap<Position, Tile>,
}

impl World {
    /// Creates a new world.
    pub fn new<I>(tiles: I) -> Self
    where
        I: Iterator<Item = (Position, Tile)>,
    {
        World {
            map: HashMap::from_iter(tiles),
        }
    }

    /// Returns a iterator over all map tiles and their position.
    pub fn tiles(&self) -> impl Iterator<Item = (&Position, &Tile)> {
        self.map.iter()
    }
}

impl Default for World {
    /// Creates a default world.
    fn default() -> Self {
        World {
            map: HashMap::new(),
        }
    }
}
