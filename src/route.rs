use ggez::graphics::{Color as ggezColor, DrawParam, Point2, Rect};
use std::{
    collections::{BinaryHeap, HashMap},
    i32::MAX,
};

use config::Config;
use draw::Drawable;
use geometry::{OrdPosition, Position};
use port::Port;
use ship::Ship;
use tile::{Tile, TileKind};
use world::World;

/// Returns all reachable tiles from a given position which a trade
/// route can pass through.
pub fn reachable(map: &[Tile], ports: &[Port], position: Position) -> Vec<Position> {
    map.iter()
        .filter(move |tile| {
            let other_position = tile.position();
            position.distance(other_position) <= 1.
                && (tile.kind() == TileKind::Water
                    || ports.iter().any(|p| p.position() == other_position))
        }).map(|tile| tile.position())
        .collect()
}

/// Finds the shortest path from start to goal using astar with
/// manhattan distance heuristic.
pub fn find_path<'a>(
    map: &[Tile],
    ports: &[Port],
    waypoints: &Vec<Waypoint>,
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
            // Add extra weight if already path exists here to avoid overlap if possible.
            let next_weight =
                weight + match waypoints.iter().any(|w| *w == Waypoint::from(position)) {
                    true => 2,
                    false => 1,
                };
            let next = OrdPosition {
                position: neighbor,
                weight: next_weight,
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

/// Represents a trading route which exists between a series of ports.
#[derive(Debug, Clone)]
pub struct Route {
    ships: Vec<Ship>,
    paths: Vec<(Position, Vec<Waypoint>)>,
}

impl Route {
    /// Creates a new empty route.
    pub fn new() -> Route {
        Route {
            ships: vec![],
            paths: vec![],
        }
    }

    /// Adds the given ship to this route.
    pub fn add_ship(&mut self, ship: Ship) {
        println!(
            "Added ship to route:\n{:#?}",
            self.paths.iter().map(|p| p.0).collect::<Vec<_>>()
        );
        self.ships.push(ship);
    }

    /// Returns an iterator over all ships on this route.
    pub fn ships(&self) -> impl Iterator<Item = &Ship> {
        self.ships.iter()
    }

    /// Returns a mutable iterator over all ships on this route.
    pub fn ships_mut(&mut self) -> impl Iterator<Item = &mut Ship> {
        self.ships.iter_mut()
    }

    /// Returns the first path.
    pub fn initial_path(&self) -> Vec<Waypoint> {
        let (_, ref path) = self.paths[0];
        path.clone()
    }

    /// Returns the next path after the given port location, return None if last.
    pub fn next_path(&self, port: Position) -> Option<Vec<Waypoint>> {
        self.paths
            .iter()
            .find(|(p, _)| *p == port)
            .map(|(_, path)| path.clone())
    }

    /// Returns the previous path after the given port location, return None if first.
    pub fn previous_path(&self, port: Position) -> Option<Vec<Waypoint>> {
        self.paths
            .iter()
            .rev()
            .skip_while(|(p, _)| *p != port)
            .nth(1)
            .map(|(_, path)| path.clone())
    }

    /// Adds a new link to this route, inserting it after start first occures.
    pub fn add_link(
        &mut self,
        map: &[Tile],
        ports: &[Port],
        waypoints: &Vec<Waypoint>,
        start: Position,
        end: Position,
        path: Vec<Waypoint>,
    ) {
        // Find the index to insert a path to the new end node.
        let index = self
            .paths
            .iter()
            .position(|(p, _)| *p == start)
            .unwrap_or(0);
        // Add empty new path.
        self.paths.insert(index, (start, path));

        // If a path already existed path from start to something else make
        // sure that, our node node leads to that old node.
        if self.paths.len() > index + 1 {
            let (port, _) = &mut self.paths[index + 1];
            *port = end;
        } else {
            self.paths.push((end, vec![]));
        }

        // Rebuild routes after.
        let new_waypoints = (index..self.paths.len() - 1)
            .map(|i| {
                let (curr_port, _) = self.paths[i];
                let (next_port, _) = self.paths[i + 1];
                let (_, route) = find_path(map, ports, &waypoints, curr_port, next_port)
                    .expect("Did not find valid route");
                route.into_iter().map(Waypoint::from).collect::<Vec<_>>()
            }).collect::<Vec<_>>();

        for (i, waypoints) in new_waypoints.into_iter().enumerate() {
            let (_, old_waypoints) = &mut self.paths[index + i];
            *old_waypoints = waypoints;
        }
    }

    /// Returns a vector with all waypoints on this route.
    pub fn waypoints(&self) -> Vec<&Waypoint> {
        self.paths
            .iter()
            .flat_map(|(_, waypoints)| waypoints.iter())
            .collect::<Vec<_>>()
    }

    /// Check if the given waypoint is contained on this route.
    pub fn contains(&self, waypoint: Waypoint) -> bool {
        self.paths
            .iter()
            .flat_map(|(_, waypoints)| waypoints.iter())
            .any(|w| *w == waypoint)
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
