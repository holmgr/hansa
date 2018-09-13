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

pub fn main() {
    // Load ggez configuration
    let c = conf::Conf {
        window_setup: conf::WindowSetup {
            samples: conf::NumSamples::Sixteen,
            ..Default::default()
        },
        ..Default::default()
    };

    // Create game context.
    let ctx = &mut Context::load_from_conf("hansa", "holmgr", c).unwrap();

    // Add resources folder to virtual filesystem.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    // Start game.
    let state = &mut gamestate::GameState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
