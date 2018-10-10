use geometry::Position;
use port::Port;
use route::{find_path, reachable, Route, RouteShape, Waypoint};
use ship::Shipyard;
use std::{collections::HashMap, iter::FromIterator};
use tally::Tally;
use tile::Tile;

/// Holds all information on the game world.
#[derive(Debug, Clone)]
pub struct World {
    map: Vec<Tile>,
    ports: Vec<Port>,
    routes: HashMap<RouteShape, Route>,
    shipyard: Shipyard,
    tally: Tally,
}

impl World {
    /// Creates a new world.
    pub fn new<I1, I2>(tiles: I1, ports: I2) -> Self
    where
        I1: Iterator<Item = Tile>,
        I2: Iterator<Item = Port>,
    {
        World {
            map: Vec::from_iter(tiles),
            ports: Vec::from_iter(ports),
            routes: HashMap::new(),
            shipyard: Shipyard::new(),
            tally: Tally::new(),
        }
    }

    /// Returns a mutable reference to the tally.
    pub fn tally_mut(&mut self) -> &mut Tally {
        &mut self.tally
    }

    /// Returns a slice over all map tiles and their position.
    pub fn tiles(&self) -> &[Tile] {
        &self.map
    }

    /// Returns a slice over all ports and their position.
    pub fn ports(&self) -> &[Port] {
        &self.ports
    }

    /// Returns a mutable slice over all ports and their position.
    pub fn ports_mut(&mut self) -> &mut [Port] {
        &mut self.ports
    }

    /// Returns a iterator over all routes.
    pub fn routes(&self) -> impl Iterator<Item = (&RouteShape, &Route)> {
        self.routes.iter()
    }

    /// Returns a mutable iterator over all routes.
    pub fn routes_mut(&mut self) -> impl Iterator<Item = (&RouteShape, &mut Route)> {
        self.routes.iter_mut()
    }

    /// Returns a mutable reference to the shipyard.
    pub fn shipyard_mut(&mut self) -> &mut Shipyard {
        &mut self.shipyard
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
        route.add_link(&self.map, &self.ports, &waypoints, start, goal, path);
    }

    /// Returns the tile at the given position.
    pub fn tile(&self, position: Position) -> Option<&Tile> {
        self.map.iter().find(|tile| tile.position() == position)
    }

    /// Returns the port at the given position.
    pub fn port(&self, position: Position) -> Option<&Port> {
        self.ports.iter().find(|port| port.position() == position)
    }

    /// Returns all reachable tiles from a given position which a trade
    /// route can pass through.
    pub fn reachable(&self, position: Position) -> Vec<Position> {
        reachable(&self.map, &self.ports, position)
    }

    /// Finds the shortest path from start to goal using astar with
    /// manhattan distance heuristic.
    pub fn route(&self, start: Position, goal: Position) -> Option<(i32, Vec<Position>)> {
        let waypoints = self
            .routes
            .values()
            .flat_map(|r| r.waypoints().into_iter().cloned())
            .collect::<Vec<_>>();
        find_path(&self.map, &self.ports, &waypoints, start, goal)
    }
}

impl Default for World {
    /// Creates a default world.
    fn default() -> Self {
        World {
            map: vec![],
            ports: vec![],
            routes: HashMap::new(),
            shipyard: Shipyard::new(),
            tally: Tally::new(),
        }
    }
}
