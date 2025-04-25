use std::f32::consts::PI;

use ::rand::rngs::ThreadRng;
use evosim::*;
use macroquad::{
    miniquad::window::{set_window_position, set_window_size},
    prelude::*,
};

fn reset(rng: &mut ThreadRng) -> World {
    // Initialise parameters
    let params = Params::default();
    let bounds = Bounds {
        x_min: 0.,
        x_max: params.window_width,
        y_min: 0.,
        y_max: params.window_height,
    };

    // Spawn in food sources
    let num_plant: usize = 10;
    let num_meat: usize = 5;
    let plant_sources: Vec<PlantSource> = (0..num_plant)
        .map(|_| PlantSource::new_rand(rng, &bounds))
        .collect::<Vec<PlantSource>>();
    let meat_sources: Vec<MeatSource> = (0..num_meat)
        .map(|_| MeatSource::new_rand(rng, &bounds))
        .collect::<Vec<MeatSource>>();

    // Spawn in creatures
    let num_creatures: usize = 20;
    let creatures: Vec<Creature> = (0..num_creatures)
        .map(|_| random_creature(rng, &bounds))
        .collect::<Vec<Creature>>();

    World::new(creatures, plant_sources, meat_sources, params, bounds)
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

fn draw_ui(x: f32, y: f32, font_size: f32, world: &World) {
    let ui_text = format!(
        "Current time = {:.2}, dt = {:.2e}, food regrow timer = {:.2}, regrow frequency = {:.2}",
        world.params.time,
        world.params.timestep,
        world.params.plant_regrow_timer,
        world.params.plant_regrow_freq
    );
    draw_text(ui_text.as_str(), x, y, font_size, BLACK);
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
    set_window_position(1000, 0);
    set_window_size(
        world.params.window_width as u32,
        world.params.window_height as u32,
    );
    // Define colors for world objects (not creatures)
    let light_blue = Color::new(0.5, 0.8, 1.0, 1.0);
    let dark_blue = Color::new(0.0, 0.2, 0.5, 1.0);
    let plant_color = Color::new(0.3, 0.7, 0.6, 1.0); // sea green
    let meat_color = Color::new(1.0, 0.6, 0.6, 1.0); // salmon
    // let mut is_paused = false;
    // Main render loop
    loop {
        // Dynamic screen sizing
        world.params.window_width = screen_width();
        world.params.window_height = screen_height();
        let params = world.params;
        // Background (clear then overwrite with ocean)
        clear_background(BLACK);
        // Smaller steps (dividing by bigger number) create finer bars
        let step = (world.params.window_height / 50.0) as usize;
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
            Color::new(0.95, 0.74, 0.15, 0.8),
        );
        //
        // Update world state
        update_world(&mut rng, &mut world);

        // Render plant sources
        for plant in world.plant_sources.values() {
            draw_circle(
                plant.position.x,
                plant.position.y,
                plant.amount / plant.max_amount * 8.,
                plant_color,
            )
        }

        // Render meat sources
        for meat in world.meat_sources.values() {
            draw_circle(
                meat.position.x,
                meat.position.y,
                meat.amount / meat.max_amount * 8.,
                meat_color,
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
        draw_ui(0., 20., 32., &world);
        world.params.plant_regrow_timer += params.timestep;
        world.params.time += params.timestep;
        next_frame().await
    }
}

/*
* TODO:
*
*/
