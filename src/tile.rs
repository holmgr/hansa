/// A map tile of a specific type.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Tile {
    Land,
    Water,
}
