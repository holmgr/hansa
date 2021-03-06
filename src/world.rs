use rand::Rng;

use geometry::Position;
use port::Port;
use route::{find_path, reachable, Route, RouteShape, Waypoint};
use ship::{Ship, Shipyard};
use std::{collections::HashMap, iter::FromIterator};
use tile::Tile;

/// Holds all information on the game world.
#[derive(Debug, Clone)]
pub struct World {
    map: Vec<Tile>,
    open_ports: Vec<Port>,
    closed_ports: Vec<Port>,
    routes: HashMap<RouteShape, Route>,
    shipyard: Shipyard,
}

impl World {
    /// Creates a new world.
    pub fn new<I1, I2>(tiles: I1, open_ports: I2, closed_ports: I2) -> Self
    where
        I1: Iterator<Item = Tile>,
        I2: Iterator<Item = Port>,
    {
        World {
            map: Vec::from_iter(tiles),
            open_ports: Vec::from_iter(open_ports),
            closed_ports: Vec::from_iter(closed_ports),
            routes: HashMap::new(),
            shipyard: Shipyard::new(),
        }
    }

    /// Opens a random closed port (if any is left), returning a mutable reference
    /// to the opened port.
    pub fn open_random_port<R: Rng>(&mut self, gen: &mut R) -> Option<&mut Port> {
        gen.shuffle(&mut self.closed_ports);
        if let Some(port) = self.closed_ports.pop() {
            self.open_ports.push(port);
            let open_ports_len = self.open_ports.len();

            // Get mutable reference to last element.
            self.open_ports.get_mut(open_ports_len - 1)
        } else {
            None
        }
    }

    /// Returns a slice over all map tiles and their position.
    pub fn tiles(&self) -> &[Tile] {
        &self.map
    }

    /// Returns a slice over all ports and their position.
    pub fn ports(&self) -> &[Port] {
        &self.open_ports
    }

    /// Returns a mutable slice over all ports and their position.
    pub fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.open_ports
    }

    /// Returns a iterator over all routes.
    pub fn routes(&self) -> impl Iterator<Item = (&RouteShape, &Route)> {
        self.routes.iter()
    }

    /// Returns a mutable iterator over all routes.
    pub fn routes_mut(&mut self) -> impl Iterator<Item = (&RouteShape, &mut Route)> {
        self.routes.iter_mut()
    }

    /// Removes all routes going through the given waypoint.
    pub fn remove_routes_at(&mut self, waypoint: Waypoint) -> Vec<Ship> {
        let mut ships = vec![];
        self.routes.retain(|_, route| {
            if route.waypoints().into_iter().any(|w| *w == waypoint) {
                ships.extend(route.remove_ships());
                false
            } else {
                true
            }
        });
        ships
    }

    /// Returns a mutable reference to the shipyard.
    pub fn shipyard_mut(&mut self) -> &mut Shipyard {
        &mut self.shipyard
    }

    /// Return all the current valid start points for drawing/extending a route.
    pub fn allowed_starts(&self, shape: RouteShape) -> Vec<Position> {
        match self.routes.get(&shape) {
            Some(route) => vec![
                route
                    .ports()
                    .cloned()
                    .last()
                    .expect("No last port location"),
            ],
            None => self
                .open_ports
                .iter()
                .map(|port| port.position())
                .collect::<Vec<_>>(),
        }
    }

    /// Return all the current valid end points for drawing/extending a route.
    pub fn allowed_ends(&self, start: Position, shape: RouteShape) -> Vec<Position> {
        match self.routes.get(&shape) {
            Some(route) => self
                .open_ports
                .iter()
                .filter_map(|port| {
                    if !route.ports().any(|p| *p == port.position()) {
                        Some(port.position())
                    } else {
                        None
                    }
                }).collect::<Vec<_>>(),
            None => self
                .open_ports
                .iter()
                .filter_map(|port| {
                    if port.position() != start {
                        Some(port.position())
                    } else {
                        None
                    }
                }).collect::<Vec<_>>(),
        }
    }

    /// Creates a new route (if non exists) and adds a new link between start and goal.
    pub fn add_route(
        &mut self,
        color: RouteShape,
        start: Position,
        goal: Position,
        path: Vec<Waypoint>,
    ) {
        let waypoints = self
            .routes
            .values()
            .flat_map(|r| r.waypoints().into_iter().cloned())
            .collect::<Vec<_>>();
        let route = self.routes.entry(color).or_insert_with(Route::new);
        route.add_link(&self.map, &self.open_ports, &waypoints, start, goal, path);
    }

    /// Returns the tile at the given position.
    pub fn tile(&self, position: Position) -> Option<&Tile> {
        self.map.iter().find(|tile| tile.position() == position)
    }

    /// Returns the port at the given position.
    pub fn port(&self, position: Position) -> Option<&Port> {
        self.open_ports
            .iter()
            .find(|port| port.position() == position)
    }

    /// Returns all reachable tiles from a given position which a trade
    /// route can pass through.
    pub fn reachable(&self, position: Position) -> Vec<Position> {
        reachable(&self.map, &self.open_ports, position)
    }

    /// Finds the shortest path from start to goal using astar with
    /// manhattan distance heuristic.
    pub fn route(&self, start: Position, goal: Position) -> Option<(i32, Vec<Position>)> {
        let waypoints = self
            .routes
            .values()
            .flat_map(|r| r.waypoints().into_iter().cloned())
            .collect::<Vec<_>>();
        find_path(&self.map, &self.open_ports, &waypoints, start, goal)
    }
}

impl Default for World {
    /// Creates a default world.
    fn default() -> Self {
        World {
            map: vec![],
            open_ports: vec![],
            closed_ports: vec![],
            routes: HashMap::new(),
            shipyard: Shipyard::new(),
        }
    }
}
