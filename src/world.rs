use color::Color;
use port::Port;
use std::{
    collections::{BinaryHeap, HashMap},
    i32::MAX,
    iter::FromIterator,
    iter::Iterator,
};
use tile::Tile;
use OrdPosition;
use Position;

/// Represents a trading route which exists between a series of ports.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Route {
    port_locations: Vec<Position>,
}

impl Route {
    /// Creates a new empty route.
    pub fn new() -> Route {
        Route {
            port_locations: vec![],
        }
    }

    /// Adds a new link to this route, inserting it after start first occures.
    pub fn add_link(&mut self, start: Position, end: Position) {
        if self.port_locations.is_empty() {
            self.port_locations.push(start);
        }

        // TODO: Will not handle case when start does not exist
        // I.e a path between two nodes, where start is not in the route already.
        if let Some(index) = self.port_locations.iter().position(|p| *p == start) {
            self.port_locations.insert(index, end);
        };
    }

    /// Returns a slice with all ports locations on this route.
    pub fn ports(&self) -> &[Position] {
        &self.port_locations[..]
    }
}

/// Holds all information on the game world.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct World {
    map: HashMap<Position, Tile>,
    ports: HashMap<Position, Port>,
    routes: HashMap<Color, Route>,
}

impl World {
    /// Creates a new world.
    pub fn new<I1, I2>(tiles: I1, ports: I2) -> Self
    where
        I1: Iterator<Item = (Position, Tile)>,
        I2: Iterator<Item = (Position, Port)>,
    {
        World {
            map: HashMap::from_iter(tiles),
            ports: HashMap::from_iter(ports),
            routes: HashMap::new(),
        }
    }

    /// Returns a iterator over all map tiles and their position.
    pub fn tiles(&self) -> impl Iterator<Item = (&Position, &Tile)> {
        self.map.iter()
    }

    /// Returns a iterator over all ports and their position.
    pub fn ports(&self) -> impl Iterator<Item = (&Position, &Port)> {
        self.ports.iter()
    }

    /// Returns a iterator over all routes.
    pub fn routes(&self) -> impl Iterator<Item = (&Color, &Route)> {
        self.routes.iter()
    }

    /// Creates a new route (if non exists) and adds a new link between start and goal.
    pub fn add_route(&mut self, color: Color, start: Position, goal: Position) {
        let route = self.routes.entry(color).or_insert_with(Route::new);
        route.add_link(start, goal);
    }

    /// Returns the tile at the given position.
    pub fn tile(&self, position: Position) -> Option<&Tile> {
        self.map.get(&position)
    }

    /// Returns the port at the given position.
    pub fn port(&self, position: Position) -> Option<&Port> {
        self.ports.get(&position)
    }

    /// Returns all reachable tiles from a given position which a trade
    /// route can pass through.
    fn reachable(&self, position: Position) -> impl Iterator<Item = &(Position)> {
        self.map
            .iter()
            .filter(move |(other_position, tile)| {
                (position.coords.x - other_position.coords.x).abs()
                    + (position.coords.y - other_position.coords.y).abs()
                    <= 1
                    && (**tile == Tile::Water || self.port(**other_position).is_some())
            }).map(|(position, _)| position)
    }

    /// Finds the shortest path from start to goal using astar with
    /// manhattan distance heuristic.
    pub fn route(&self, start: Position, goal: Position) -> Option<(i32, Vec<Position>)> {
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
            for neighbor in self.reachable(position) {
                let next = OrdPosition {
                    weight: weight + 1,
                    position: *neighbor,
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
}

impl Default for World {
    /// Creates a default world.
    fn default() -> Self {
        World {
            map: HashMap::new(),
            ports: HashMap::new(),
            routes: HashMap::new(),
        }
    }
}
