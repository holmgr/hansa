/// A map tile of a specific type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Land,
    Water,
}
