use std::f32::consts::PI;

use ::rand::rngs::ThreadRng;
use evosim::*;
use macroquad::prelude::*;

fn reset(rng: &mut ThreadRng) -> World {
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

    World::new(creatures, food_sources, params, bounds)
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

fn lerp_color(c1: Color, c2: Color, s: f32) -> Color {
    // Lerps between two colors; v should be a f32 between 0 and 1 (inclusive)
    // that is the percent between c1 and c2
    let v1 = Vec4::new(c1.r, c1.g, c1.b, c1.a);
    let v2 = Vec4::new(c2.r, c2.g, c2.b, c2.a);
    let v3 = v1.lerp(v2, s);
    Color::new(v3.x, v3.y, v3.z, v3.w)
}

#[macroquad::main("EvoSim")]
async fn main() {
    // Initial setup
    let mut rng: ThreadRng = ::rand::rng();
    let mut world = reset(&mut rng);
    // let mut is_paused = false;
    // Main render loop
    loop {
        // Dynamic screen sizing
        world.params.window_width = screen_width();
        world.params.window_height = screen_height();
        let params = world.params;
        // Background (clear then overwrite with ocean)
        clear_background(BLACK);
        let light_blue = Color::new(0.5, 0.8, 1.0, 1.0);
        let dark_blue = Color::new(0.0, 0.2, 0.5, 1.0);
        let step = (world.params.window_height / 20.0) as usize;
        // Draw a rectangle of step-px lines by interpolating lightblue -> darkblue
        for y in (0..params.window_height as i32).step_by(step) {
            let color = lerp_color(light_blue, dark_blue, y as f32 / params.window_height);
            draw_line(
                0.0,
                y as f32,
                params.window_width,
                y as f32,
                step as f32,
                color,
            );
        }
        // Update last line (otherwise will be black)
        draw_line(
            0.0,
            params.window_height,
            params.window_width,
            params.window_height,
            step as f32,
            dark_blue,
        );
        //
        // Update world state
        update_world(&mut rng, &mut world);

        // Render food sources
        for food in world.food_sources.values() {
            draw_circle(
                food.position.x,
                food.position.y,
                food.amount / food.max_amount * 8.,
                GREEN,
            )
        }

        // Render creatures
        for creature in world.creatures.values() {
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
