use ggez::graphics::{DrawParam, Point2, Rect};

use draw::Drawable;
use geometry::Position;
use world::World;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port {
    position: Position,
}

impl Port {
    /// Creates a new port.
    pub fn new(position: Position) -> Self {
        Port { position }
    }

    /// Returns the position.
    pub fn position(self) -> Position {
        self.position
    }
}

impl<'a> Drawable<'a> for Port {
    type Data = World;

    fn draw(&self, _world: &World) -> DrawParam {
        DrawParam {
            src: Rect::new(0., 2. * Self::TILE_OFFSET, Self::TILE_SIZE, Self::TILE_SIZE),
            dest: Point2::from(self.position),
            ..Default::default()
        }
    }
}
