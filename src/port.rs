use ggez::{
    graphics::{Color as ggezColor, DrawParam, Point2, Rect},
    timer::get_ticks,
    Context,
};
use rand::{seq::sample_slice, Rng, ThreadRng};

use color::Color;
use draw::Drawable;
use geometry::Position;
use update::Updatable;
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
            export: initial_colors[1],
        }
    }

    /// Returns the position.
    pub fn position(self) -> Position {
        self.position
    }

    /// Returns the current import color.
    pub fn import(&self) -> Color {
        self.import
    }

    /// Returns the current export color.
    pub fn export(&self) -> Color {
        self.export
    }
}

impl<'a> Updatable<'a> for Port {
    type Data = &'a mut ThreadRng;

    fn update(&mut self, ctx: &Context, gen: &'a mut ThreadRng) {
        //let seconds_passed = get_time_since_start(ctx).as_secs();
        let ticks = get_ticks(ctx);
        // TODO: Move magic constant for color switching.
        if ticks % 100 == 0 && gen.gen_bool(1. / 10.) {
            // TODO: Make swtiching more elegant and handle cases where we end up with no red etc.
            let new_colors = sample_slice(gen, &Color::values(), 2);
            self.import = new_colors[0];
            self.export = new_colors[1];
        }
    }
}

impl<'a> Drawable<'a> for Port {
    type Data = World;

    fn draw(&self, _world: &World) -> Vec<DrawParam> {
        let (e_r, e_g, e_b) = self.export.rgb();
        let (i_r, i_g, i_b) = self.import.rgb();
        vec![
            DrawParam {
                src: Rect::new(
                    Self::TILE_OFFSET,
                    2. * Self::TILE_OFFSET,
                    Self::TILE_SIZE,
                    Self::TILE_SIZE,
                ),
                dest: Point2::from(self.position),
                color: Some(ggezColor::from_rgb(i_r, i_g, i_b)),
                ..Default::default()
            },
            DrawParam {
                src: Rect::new(0., 2. * Self::TILE_OFFSET, Self::TILE_SIZE, Self::TILE_SIZE),
                dest: Point2::from(self.position),
                color: Some(ggezColor::from_rgb(e_r, e_g, e_b)),
                ..Default::default()
            },
        ]
    }
}
