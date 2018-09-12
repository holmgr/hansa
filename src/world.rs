use std::{collections::HashMap, iter::FromIterator};
use tile::Tile;

use Position;

/// Holds all information on the game world.
pub struct World {
    map: HashMap<Position, Tile>,
}

impl World {
    /// Creates a new world.
    fn new<I>(tiles: I) -> Self
    where
        I: Iterator<Item = (Position, Tile)>,
    {
        World {
            map: HashMap::from_iter(tiles),
        }
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
