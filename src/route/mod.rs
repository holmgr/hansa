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

mod routebuilder;
mod shape;
mod waypoint;
pub use self::routebuilder::RouteBuilder;
pub use self::shape::{RouteShape, ShapeSelector};
pub use self::waypoint::Waypoint;

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
pub fn find_path(
    map: &[Tile],
    ports: &[Port],
    waypoints: &[Waypoint],
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
            let next_weight = weight
                + if waypoints.iter().any(|w| *w == Waypoint::from(position)) {
                    2
                } else {
                    1
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

    /// Returns an iterator over all port locations on this path in order.
    pub fn ports(&self) -> impl Iterator<Item = &Position> {
        self.paths.iter().map(|(p, _)| p)
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

    /// Returns the path which contains the given waypoint.
    /// If given a port position it will return the path leading to that one.
    pub fn path(&self, waypoint: Waypoint) -> Vec<Waypoint> {
        let (_, path) = self
            .paths
            .iter()
            .cloned()
            .find(|(_, path)| path.iter().any(|w| *w == waypoint))
            .unwrap();
        path
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
        waypoints: &[Waypoint],
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
