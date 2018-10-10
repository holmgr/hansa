use super::*;

/// Manages the placement of ships on routes.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ShipBuilder;

impl ShipBuilder {
    /// Create a new ship builder.
    pub fn new() -> Self {
        ShipBuilder {}
    }

    /// Attempts to place a ship at the given position, consuming the ship builder.
    /// Returns the builder if it failed.
    pub fn try_place(self, position: Position, world: &mut World) -> Option<ShipBuilder> {
        // TODO: Implement placement on the given possition if it exists a
        // valid path there.

        let waypoint = Waypoint::from(position);
        if let Some((_, route)) = world.routes_mut().find(|(_, r)| r.contains(waypoint)) {
            println!("Placed ship on route!");
            let initial_path = route.path(waypoint);
            route.add_ship(Ship::new(waypoint, initial_path));
            None
        } else {
            Some(self)
        }
    }
}

impl<'a> Drawable<'a> for ShipBuilder {
    type Data = Point2; // Mouse position

    fn draw(&self, mouse: &Point2) -> Vec<DrawParam> {
        vec![DrawParam {
            src: Rect::new(
                3. * Self::TILE_SIZE,
                2. * Self::TILE_SIZE,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(mouse.coords.x, mouse.coords.y),
            offset: Point2::new(0.5, 0.5),
            color: Some(ggezColor::from_rgb(69, 55, 52)),
            ..Default::default()
        }]
    }
}
