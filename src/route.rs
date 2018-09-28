use ggez::graphics::{Color as ggezColor, DrawParam, Point2, Rect};
use std::{
    collections::{BinaryHeap, HashMap},
    i32::MAX,
};

use color::Color;
use draw::Drawable;
use port::Port;
use tile::{Tile, TileKind};
use world::World;
use OrdPosition;
use Position;

/// Returns all reachable tiles from a given position which a trade
/// route can pass through.
pub fn reachable(map: &[Tile], ports: &[Port], position: Position) -> Vec<Position> {
    map.iter()
        .filter(move |tile| {
            let other_position = tile.position();
            (position.coords.x - other_position.coords.x).abs()
                + (position.coords.y - other_position.coords.y).abs()
                <= 1
                && (tile.kind() == TileKind::Water
                    || ports.iter().any(|p| p.position() == other_position))
        }).map(|tile| tile.position())
        .collect()
}

/// Finds the shortest path from start to goal using astar with
/// manhattan distance heuristic.
pub fn find_path(
    map: &[Tile],
    ports: &[Port],
    start: Position,
    goal: Position,
) -> Option<(i32, Vec<Position>)> {
    // Node -> steps, cost mapping.
    let mut dist = HashMap::<Position, i32>::new();
    let mut frontier = BinaryHeap::new();
    let mut previous = HashMap::<Position, Position>::new();

    // We're at `start`, with a zero cost
    dist.insert(start, 0);
    frontier.push(OrdPosition {
        weight: 0,
        position: start,
    });

    let mut cost = None;
    // Examine the frontier with lower cost nodes first (min-heap)
    while let Some(OrdPosition { position, weight }) = frontier.pop() {
        // Alternatively we could have continued to find all shortest paths
        if position == goal {
            cost = Some(weight);
            break;
        }

        // Important as we may have already found a better way
        if weight > *dist.get(&position).unwrap_or(&MAX) {
            continue;
        }

        // For each node we can reach, see if we can find a way with
        // a lower cost going through this node
        for neighbor in reachable(map, ports, position) {
            let next = OrdPosition {
                weight: weight + 1,
                position: neighbor,
            };

            // If so, add it to the frontier and continue
            if next.weight < *dist.get(&next.position).unwrap_or(&MAX) {
                // Relaxation, we have now found a better way
                dist.insert(next.position, next.weight);
                previous.insert(next.position, position);
                frontier.push(next);
            }
        }
    }

    match cost {
        Some(cost) => {
            let mut path = vec![];
            let mut current = goal;
            while current != start {
                path.push(current);
                current = previous.remove(&current).unwrap();
            }
            path.push(start);
            path.reverse();
            Some((cost, path))
        }
        None => None,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
/// A waypoint is a drawable position.
pub struct Waypoint(Position);

impl<'a> Drawable<'a> for Waypoint {
    type Data = Color;

    fn draw(&self, color: &Color) -> DrawParam {
        let (r, g, b) = color.rgb();
        DrawParam {
            src: Rect::new(
                Self::TILE_OFFSET,
                2. * Self::TILE_OFFSET,
                Self::TILE_SIZE,
                Self::TILE_SIZE,
            ),
            dest: Point2::new(self.0.coords.x as f32, self.0.coords.y as f32),
            color: Some(ggezColor::from_rgb(r, g, b)),
            ..Default::default()
        }
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

/// Represents a trading route which exists between a series of ports.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Route(Vec<(Position, Vec<Waypoint>)>);

impl Route {
    /// Creates a new empty route.
    pub fn new() -> Route {
        Route(vec![])
    }

    /// Adds a new link to this route, inserting it after start first occures.
    pub fn add_link(
        &mut self,
        map: &[Tile],
        ports: &[Port],
        start: Position,
        end: Position,
        path: Vec<Waypoint>,
    ) {
        // Find the index to insert a path to the new end node.
        let index = self.0.iter().position(|(p, _)| *p == start).unwrap_or(0);
        // Add empty new path.
        self.0.insert(index, (start, path));

        // If a path already existed path from start to something else make
        // sure that, our node node leads to that old node.
        if let Some((port, _)) = self.0.get_mut(index + 1) {
            *port = end;
        }

        // Rebuild routes after.
        let new_waypoints = (index..self.0.len() - 1)
            .map(|i| {
                let (curr_port, _) = self.0[i];
                let (next_port, _) = self.0[i + 1];
                let (_, route) =
                    find_path(map, ports, curr_port, next_port).expect("Did not find valid route");
                route.into_iter().map(Waypoint::from).collect::<Vec<_>>()
            }).collect::<Vec<_>>();

        for (i, waypoints) in new_waypoints.into_iter().enumerate() {
            let (_, old_waypoints) = &mut self.0[index + i];
            *old_waypoints = waypoints;
        }
    }

    /// Returns a vector with all waypoints on this route.
    pub fn waypoints(&self) -> Vec<&Waypoint> {
        self.0
            .iter()
            .flat_map(|(_, waypoints)| waypoints.iter())
            .collect::<Vec<_>>()
    }
}

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