use std::iter::FromIterator;

use color::Color;

/// Keeps track of the amount of each color collected.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tally {
    collected: Vec<(Color, u32)>,
}

impl Tally {
    /// Creates a new tally.
    pub fn new() -> Self {
        Tally {
            collected: Vec::from_iter(Color::values().into_iter().map(|c| (c, 0))),
        }
    }

    /// Returns the current score.
    pub fn score(&self) -> u32 {
        self.collected
            .iter()
            .map(|(_, amount)| *amount)
            .min()
            .expect("No score found")
    }

    /// Returns the amount collected for the given color.
    pub fn get(&self, color: Color) -> u32 {
        self.collected
            .iter()
            .find(|(c, _)| *c == color)
            .map(|(_, amount)| *amount)
            .expect("Tally for color not found")
    }

    /// Increments the tally for the given color.
    pub fn update(&mut self, color: Color) {
        let (_, amount) = self
            .collected
            .iter_mut()
            .find(|(c, _)| *c == color)
            .expect("Tally for color not found");
        *amount += 1;
    }
}
