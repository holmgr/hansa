use ggez::graphics::{DrawParam, Point2, Rect};

use draw::Drawable;
use geometry::Position;
use world::World;

/// Sepecific Tile type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Land,
    Water,
}

/// A map tile of a specific type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    position: Position,
    kind: TileKind,
}

impl Tile {
    /// Creates a new tile.
    pub fn new(position: Position, kind: TileKind) -> Self {
        Tile { position, kind }
    }

    /// Returns the tile kind.
    pub fn kind(&self) -> TileKind {
        self.kind
    }

    /// Returns the position.
    pub fn position(&self) -> Position {
        self.position
    }

    /// Returns a list of all neighbors (north, east, south, west).
    pub fn neighbors(&self) -> [Position; 4] {
        let (x, y) = (self.position.x, self.position.y);
        [
            Position::new(x, y - 1),
            Position::new(x + 1, y),
            Position::new(x, y + 1),
            Position::new(x - 1, y),
        ]
    }
}

impl<'a> Drawable<'a> for Tile {
    type Data = World;

    fn draw(&self, world: &World) -> DrawParam {
        let neighbors: Vec<Option<TileKind>> = self
            .neighbors()
            .iter()
            .map(|position| world.tile(*position).map(|tile| tile.kind()))
            .collect();

        // TODO: Do something about this ugly code.
        let (north_tile, east_tile, south_tile, west_tile) =
            (neighbors[0], neighbors[1], neighbors[2], neighbors[3]);
        let curr_tile = self.kind();
        let src = match (curr_tile, north_tile, east_tile, south_tile, west_tile) {
            (TileKind::Water, _, _, _, _) => Rect::new(0., 0., Self::TILE_SIZE, Self::TILE_SIZE),
            (
                TileKind::Land,
                Some(TileKind::Water),
                Some(TileKind::Water),
                Some(TileKind::Land),
                Some(TileKind::Land),
            ) => Rect::new(0.0, Self::TILE_OFFSET, Self::TILE_SIZE, Self::TILE_SIZE),
            (
                TileKind::Land,
                Some(TileKind::Land),
                Some(TileKind::Water),
                Some(TileKind::Water),
                Some(TileKind::Land),
            ) => Rect::new(
                1. * Self::TILE_OFFSET,
                Self::TILE_OFFSET,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            (
                TileKind::Land,
                Some(TileKind::Land),
                Some(TileKind::Land),
                Some(TileKind::Water),
                Some(TileKind::Water),
            ) => Rect::new(
                2. * Self::TILE_OFFSET,
                Self::TILE_OFFSET,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            (
                TileKind::Land,
                Some(TileKind::Water),
                Some(TileKind::Land),
                Some(TileKind::Land),
                Some(TileKind::Water),
            ) => Rect::new(
                3. * Self::TILE_OFFSET,
                Self::TILE_OFFSET,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            (TileKind::Land, _, _, _, _) => {
                Rect::new(Self::TILE_OFFSET, 0., Self::TILE_SIZE, Self::TILE_SIZE)
            }
        };

        let dest = Point2::from(self.position);
        DrawParam {
            src,
            dest,
            ..Default::default()
        }
    }
}
