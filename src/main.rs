#[macro_use]
extern crate serde_derive;
extern crate ggez;
extern crate nalgebra as na;
extern crate serde;
extern crate serde_json;
use ggez::{conf, event, Context};
use std::{env, path::PathBuf, cmp::Ordering};

mod gamestate;
pub mod tile;
pub mod world;
pub mod port;
pub mod config;
pub mod path;

/// General position type.
pub type Position = na::Point2<i32>;

/// Ordered position based on weight.
pub struct OrdPosition {
    pub position: Position,
    pub weight: i32
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


static GAME_ID: &str = "hansa";
static AUTHOR: &str = "holmgr";

/// Attempts to load game context.
fn load_context() -> Context {
    let mut default_conf = conf::Conf::new();
    default_conf.window_mode.fullscreen_type = conf::FullscreenType::Off;
    default_conf.window_setup.samples = conf::NumSamples::Sixteen;
    default_conf.window_setup.resizable = true;
    default_conf.window_setup.allow_highdpi = false; // Diallow due to scaling bug.
    Context::load_from_conf(GAME_ID, AUTHOR, default_conf).unwrap()
}

/// Attempt to set the correct screen resolution.
fn set_optimal_resolution(ctx: &mut Context) {
    if let Ok(screen_modes) = ggez::graphics::get_fullscreen_modes(&ctx, 0) {
        let (width, height) = screen_modes[0];
        println!("{},{}", width, height);
        ggez::graphics::set_mode(
            ctx,
            conf::WindowMode {
                width,
                height,
                fullscreen_type: conf::FullscreenType::Desktop,
                ..Default::default()
            },
        ).unwrap();
        ggez::graphics::set_screen_coordinates(
            ctx,
            ggez::graphics::Rect::new_i32(0, 0, width as i32, height as i32),
        ).unwrap();
    }
}

pub fn main() {

    // Initialize default config, set specified parameters from command line.
    let mut config = config::Config::default();
    let args: Vec<String> = env::args().collect();
    if args.iter().find(|ref arg| arg.as_str() == "--fuck-apple").is_some() {
        config.scaling = 2;
    }

    // Create game context.
    let mut ctx = load_context();

    // Add resources folder to virtual filesystem.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }
    set_optimal_resolution(&mut ctx);

    // Start game.
    let state = &mut gamestate::GameState::new(&mut ctx, config).unwrap();
    if let Err(e) = event::run(&mut ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
