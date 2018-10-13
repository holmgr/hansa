use ggez::graphics::{Color as ggezColor, DrawParam, Point2, Rect};
use rand::{seq::sample_slice, Rng};

use animation::Animation;
use color::Color;
use draw::Drawable;
use geometry::Position;
use world::World;

/// Returns whether the given amount of ports is a valid configuration of imports/exports.
pub fn is_valid_arrangement(ports: &[Port]) -> bool {
    let (mut imports, mut exports) = (Color::values(), Color::values());
    for port in ports {
        let (import, export) = (port.import(), port.export());

        // Same import, export color is not valid.
        if import == export {
            return false;
        }
        if let Some(index) = imports.iter().position(|c| *c == import) {
            imports.swap_remove(index);
        }
        if let Some(index) = exports.iter().position(|c| *c == export) {
            exports.swap_remove(index);
        }
    }
    imports.is_empty() && exports.is_empty()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port {
    import: Color,
    export: Color,
    position: Position,
    animation: Option<Animation>,
}

impl Port {
    /// Creates a new port.
    pub fn new<R: Rng>(position: Position, gen: &mut R) -> Self {
        let (import, export) = Port::sample_colors(gen);
        Port {
            position,
            import,
            export,
            animation: None,
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

    /// Returns a mutable reference to the current import color.
    pub fn import_mut(&mut self) -> &mut Color {
        &mut self.import
    }

    /// Returns a mutable reference to the current export color.
    pub fn export_mut(&mut self) -> &mut Color {
        &mut self.export
    }

    /// Samples a random import and export color.
    pub fn sample_colors<R: Rng>(gen: &mut R) -> (Color, Color) {
        let colors = sample_slice(gen, &Color::values(), 2);
        (colors[0], colors[1])
    }

    pub fn animation_mut(&mut self) -> &mut Option<Animation> {
        &mut self.animation
    }
}

impl<'a> Drawable<'a> for Port {
    type Data = World;

    fn animation(&self) -> Option<Animation> {
        self.animation
    }

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
                dest: Point2::new(self.position.x as f32 + 0.5, self.position.y as f32 + 0.5),
                color: Some(ggezColor::from_rgb(i_r, i_g, i_b)),
                offset: Point2::new(0.5, 0.5),
                ..Default::default()
            },
            DrawParam {
                src: Rect::new(0., 2. * Self::TILE_OFFSET, Self::TILE_SIZE, Self::TILE_SIZE),
                dest: Point2::new(self.position.x as f32 + 0.5, self.position.y as f32 + 0.5),
                color: Some(ggezColor::from_rgb(e_r, e_g, e_b)),
                offset: Point2::new(0.5, 0.5),
                ..Default::default()
            },
        ]
    }
}
