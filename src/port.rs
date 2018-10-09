use ggez::graphics::{DrawParam, Point2, Rect};
use rand::{seq::sample_slice, Rng};

use color::Color;
use draw::Drawable;
use geometry::Position;
use world::World;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port {
    import: Color,
    export: Color,
    position: Position,
}

impl Port {
    /// Creates a new port.
    pub fn new<R: Rng>(position: Position, gen: &mut R) -> Self {
        let initial_colors = sample_slice(gen, &Color::values(), 2);
        println!("Got intial colors: {:#?} at {:?}", initial_colors, position);
        Port { 
            position,
            import: initial_colors[0],
            export: initial_colors[1]
        }
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
