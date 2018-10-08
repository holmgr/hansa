use ggez::graphics::Point2;
use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A point in two dimensional space.
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Creates a new position with the given coordinates.
    pub fn new(x: i32, y: i32) -> Self {
        Position { x, y }
    }

    /// Returns the euclidean distance between this and another point.
    pub fn distance(self, other: Position) -> f32 {
        ((self.x - other.x).pow(2) as f32 + (self.y - other.y).pow(2) as f32).sqrt()
    }

    /// Returns the distance from this point to origo.
    pub fn distance_origo(self) -> f32 {
        self.distance(Position::default())
    }

    /// Returns the direction as an angle, in radians, between origo and the point.
    pub fn direction(self) -> f32 {
        (self.y as f32 / self.x as f32).atan()
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position::new(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, other: Position) {
        *self = Position::new(self.x + other.x, self.y + other.y);
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, other: Position) -> Position {
        Position::new(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, other: Position) {
        *self = Position::new(self.x - other.x, self.y - other.y);
    }
}

impl Mul<f32> for Position {
    type Output = Position;

    fn mul(self, rhs: f32) -> Position {
        Position::new((self.x as f32 * rhs) as i32, (self.y as f32 * rhs) as i32)
    }
}

impl MulAssign<f32> for Position {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Default for Position {
    /// Creates a new position at origo.
    fn default() -> Self {
        Position::new(0, 0)
    }
}

impl From<Point2> for Position {
    fn from(position: Point2) -> Self {
        Position::new(position.coords.x as i32, position.coords.y as i32)
    }
}

impl From<Position> for Point2 {
    fn from(position: Position) -> Self {
        Point2::new(position.x as f32, position.y as f32)
    }
}

/// Ordered position based on weight.
pub struct OrdPosition {
    pub position: Position,
    pub weight: i32,
}

impl Ord for OrdPosition {
    fn cmp(&self, other: &OrdPosition) -> Ordering {
        other.weight.partial_cmp(&self.weight).unwrap()
    }
}

impl PartialOrd for OrdPosition {
    fn partial_cmp(&self, other: &OrdPosition) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OrdPosition {
    fn eq(&self, other: &OrdPosition) -> bool {
        self.weight == other.weight
    }
}

impl Eq for OrdPosition {}
