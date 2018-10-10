use super::*;

/// Represents the visual display of the route path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteShape {
    Plus,
    Cross,
    Star,
}

impl RouteShape {
    /// Returns all shape variants.
    pub fn values() -> Vec<RouteShape> {
        vec![RouteShape::Plus, RouteShape::Cross, RouteShape::Star]
    }
}

impl<'a> Drawable<'a> for RouteShape {
    type Data = (&'a Config, &'a ShapeSelector);

    fn draw(&self, data: &(&Config, &ShapeSelector)) -> Vec<DrawParam> {
        let (config, selector) = data;
        let index = selector.find(*self).unwrap();

        // Add little scale factor to indicate do the user which shape is selected.
        let scale_factor = match selector.selected() {
            Some(selected_shape) if selected_shape == *self => 2.8,
            _ => 1.8,
        };
        let num_shapes = selector.shapes().count() as u32;
        let shape_selector_x_offset = (config.grid_width / 2 - num_shapes + 1) as f32;
        let shape_selector_y_offset = (config.grid_height as f32 + 1.) as f32;

        let x_offset = match self {
            RouteShape::Plus => 0.,
            RouteShape::Cross => 1.,
            RouteShape::Star => 2.,
        } * Self::TILE_OFFSET;

        vec![DrawParam {
            src: Rect::new(
                x_offset,
                3. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(
                shape_selector_x_offset + 2. * index as f32 as f32,
                shape_selector_y_offset,
            ),
            scale: Point2::new(scale_factor, scale_factor),
            offset: Point2::new(0.5, 0.5),
            color: Some(ggezColor::from_rgb(0, 0, 0)), // TODO: Change placeholder color.
            ..Default::default()
        }]
    }
}

/// Handles selection, unselection of a series of shapes.
pub struct ShapeSelector {
    selected: Option<usize>,
    shapes: Vec<RouteShape>,
}

impl ShapeSelector {
    /// Create a new shape seclector.
    pub fn new() -> Self {
        ShapeSelector {
            selected: None,
            shapes: RouteShape::values(),
        }
    }

    /// Returns the selected shape, if any.
    pub fn selected(&self) -> Option<RouteShape> {
        if let Some(index) = self.selected {
            Some(self.shapes[index])
        } else {
            None
        }
    }

    /// Selects the shape at the given index if not already selected,
    /// otherwise unselect.
    pub fn toggle(&mut self, index: usize) {
        self.selected = match self.selected {
            Some(current_index) if current_index == index => None,
            Some(_) => Some(index),
            None => Some(index),
        };
    }

    /// Returns the index of the given shape.
    pub fn find(&self, shape: RouteShape) -> Option<usize> {
        self.shapes.iter().position(|c| *c == shape)
    }

    /// Returns an iterator over all shapes available in this selector.
    pub fn shapes(&self) -> impl Iterator<Item = &RouteShape> {
        self.shapes.iter()
    }
}
