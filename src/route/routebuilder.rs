use super::*;

/// Manages a route in creation/extension.
pub struct RouteBuilder {
    from: Position,
    to: Position,
    path: Option<Vec<Waypoint>>,
}

impl RouteBuilder {
    /// Create a new route builder.
    pub fn new(from: Position) -> Self {
        RouteBuilder {
            from,
            to: from,
            path: None,
        }
    }

    /// Updates the best path based on the end point given.
    pub fn update(&mut self, new_to: Position, world: &World) {
        // Only update if end position has changed.
        if new_to != self.to {
            self.to = new_to;
            self.path = world
                .route(self.from, self.to)
                .map(|(_, path)| path.into_iter().map(Waypoint::from).collect());
        }
    }

    /// Returns the current path if any
    pub fn path(&self) -> &Option<Vec<Waypoint>> {
        &self.path
    }

    /// Returns the starting position.
    pub fn from(&self) -> &Position {
        &self.from
    }

    /// Returns the end position.
    pub fn to(&self) -> &Position {
        &self.to
    }
}
