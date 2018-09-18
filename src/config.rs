/// Holds game specific configurations.
pub struct Config {
    pub scaling: u32, // Scaling needs to be handled due to Apples OpenGL implementation.
}

impl Default for Config {
    fn default() -> Config {
        Config { scaling: 1 }
    }
}
