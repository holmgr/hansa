#[macro_use]
extern crate serde_derive;
extern crate ggez;
extern crate nalgebra as na;
extern crate serde;
extern crate serde_json;
use ggez::{conf, event, Context};
use std::{env, path};

mod gamestate;
pub mod tile;
pub mod world;

pub type Position = na::Point2<i32>;

static GAME_ID: &str = "hansa";
static AUTHOR: &str = "holmgr";

/// Attempts to load game context.
fn load_context() -> Context {
    let mut default_conf = conf::Conf::new();
    default_conf.window_mode.fullscreen_type = conf::FullscreenType::Off;
    default_conf.window_setup.samples = conf::NumSamples::Sixteen;
    default_conf.window_setup.resizable = true;
    default_conf.window_setup.allow_highdpi = true;
    Context::load_from_conf(GAME_ID, AUTHOR, default_conf).unwrap()
}

pub fn main() {
    // Create game context.
    let mut ctx = load_context();

    // Add resources folder to virtual filesystem.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    // Start game.
    let state = &mut gamestate::GameState::new(&mut ctx).unwrap();
    if let Err(e) = event::run(&mut ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
