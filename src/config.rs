/// Holds game specific configurations.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub scaling: u32, // Scaling needs to be handled due to Apples OpenGL implementation.
    pub grid_width: u32, // Width of the grid.
    pub grid_height: u32, // Height of the grid.
}

impl Default for Config {
    fn default() -> Config {
        Config {
            scaling: 1,
            grid_width: 60,
            grid_height: 30,
        }
    }
}
