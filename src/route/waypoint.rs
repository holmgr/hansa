use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A waypoint is a drawable position.
pub struct Waypoint(Position);

impl<'a> Drawable<'a> for Waypoint {
    type Data = RouteShape;

    fn draw(&self, shape: &RouteShape) -> Vec<DrawParam> {
        let x_offset = match shape {
            RouteShape::Plus => 0.,
            RouteShape::Cross => 1.,
            RouteShape::Star => 2.,
        } * Self::TILE_OFFSET;
        vec![DrawParam {
            src: Rect::new(
                x_offset,
                3. * Self::TILE_OFFSET,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::from(self.0),
            color: Some(ggezColor::from_rgb(0, 0, 0)), // TODO: Change placeholder color.
            ..Default::default()
        }]
    }
}

impl From<Position> for Waypoint {
    fn from(position: Position) -> Self {
        Waypoint(position)
    }
}

impl From<Waypoint> for Position {
    fn from(waypoint: Waypoint) -> Self {
        waypoint.0
    }
}
