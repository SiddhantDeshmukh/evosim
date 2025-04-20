use std::f32::consts::PI;

use ::rand::rngs::ThreadRng;
use evosim::*;
use macroquad::prelude::*;

fn reset(rng: &mut ThreadRng) -> (Params, Bounds, Vec<FoodSource>, Vec<Creature>) {
    // Initialise parameters
    let params = Params::default();
    let bounds = Bounds {
        x_min: 0.,
        x_max: screen_width(),
        y_min: 0.,
        y_max: screen_height(),
    };

    let num_food: usize = 10;
    let num_creatures: usize = 5;
    let food_sources: Vec<FoodSource> = (0..num_food)
        .map(|_| FoodSource::new_rand(rng, &bounds))
        .collect::<Vec<FoodSource>>();
    let creatures: Vec<Creature> = (0..num_creatures)
        .map(|_| random_creature(rng, &bounds))
        .collect::<Vec<Creature>>();

    (params, bounds, food_sources, creatures)
}

fn draw_fps(x: f32, y: f32, font_size: f32) {
    let fps = get_fps();
    let c = match fps {
        0..=10 => RED,
        11..=30 => ORANGE,
        _ => GREEN,
    };
    draw_text(format!("FPS: {}", fps).as_str(), x, y, font_size, c);
}

#[macroquad::main("EvoSim")]
async fn main() {
    // Initial setup
    let mut rng: ThreadRng = ::rand::rng();
    let (mut params, bounds, mut food_sources, mut creatures) = reset(&mut rng);
    let mut is_paused = false;
    // Main render loop
    loop {
        // Dynamic screen sizing
        params.window_width = screen_width();
        params.window_height = screen_height();
        // Background
        clear_background(BLACK);
        // Update world state
        (food_sources, creatures) =
            update_world(&mut rng, &params, &bounds, &food_sources, &creatures);

        // Render food sources
        for food in food_sources.iter() {
            draw_circle(
                food.position.x,
                food.position.y,
                food.amount / food.max_amount * 8.,
                GREEN,
            )
        }

        // Render creatures
        for creature in creatures.iter() {
            draw_poly(
                creature.position.x,
                creature.position.y,
                3,
                6.,
                creature.facing * 180. / PI,
                creature.color,
            );
        }

        // Final draw, move to next frame
        draw_fps(params.window_width - 120., 20., 32.);
        next_frame().await
    }
}
