/// Holds game specific configurations.
pub struct Config {
    pub scaling: u32, // Scaling needs to be handled due to Apples OpenGL implementation.
    pub grid_width: u32, // Width of the grid.
    pub grid_height: u32, // Height of the grid.
}

impl Default for Config {
    fn default() -> Config {
        Config {
            scaling: 1,
            grid_width: 64,
            grid_height: 32,
        }
    }
}
