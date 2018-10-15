#[macro_use]
extern crate serde_derive;
extern crate ggez;
extern crate rand;
extern crate serde;
extern crate serde_json;

pub mod animation;
pub mod audio;
pub mod color;
pub mod config;
pub mod draw;
pub mod fonts;
mod gamestate;
pub mod geometry;
pub mod menustate;
pub mod port;
pub mod route;
pub mod scorestate;
pub mod ship;
pub mod tally;
pub mod tile;
pub mod time;
pub mod update;
pub mod world;

use ggez::{conf, event, Context};
use std::{cell::RefCell, env, path::PathBuf};

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

pub fn main() -> ggez::GameResult<()> {
    // Initialize default config, set specified parameters from command line.
    let mut config = config::Config::default();
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|ref arg| arg.as_str() == "--fuck-apple") {
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

    // Start main menu.
    let menu_state = &mut menustate::MenuState::new(&mut ctx, config)?;

    // Run main menu until clean exit.
    event::run(&mut ctx, menu_state)?;

    let tally = RefCell::new(tally::Tally::new());

    // Start game, run until completion.
    let game_state = &mut gamestate::GameState::new(&mut ctx, config, &tally)?;
    event::run(&mut ctx, game_state)?;

    // Start score state.
    let score_state = &mut scorestate::ScoreState::new(&mut ctx, &tally)?;
    let result = event::run(&mut ctx, score_state);
    println!("{:?}", result);
    result
}
