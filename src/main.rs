extern crate ggez;
use ggez::{conf, event, Context};

mod gamestate;

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

    // Start game.
    let state = &mut gamestate::GameState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
