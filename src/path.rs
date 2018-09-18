use world::World;
use Position;

type Path = (i32, Vec<Position>);

pub struct PathDrawer {
    from: Position,
    to: Position,
    path: Option<Path>
}

impl PathDrawer {
    pub fn new(from: Position) -> Self {
        PathDrawer { from, to: from, path: None }
    }

    pub fn update(&mut self, new_to: Position, world: &World) {
        // Only update if end position has changed.
        if new_to != self.to {
            self.to = new_to;
            self.path = world.route(self.from, self.to);
        }
    }

    pub fn path(&self) -> &Option<Path> {
        &self.path
    }
}
