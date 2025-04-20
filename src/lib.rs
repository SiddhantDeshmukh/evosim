use std::{collections::HashMap, f32::consts::PI};

use ::rand::{Rng, rngs::ThreadRng};
use macroquad::prelude::*;

// Traits
pub trait HasPosition {
    fn position(&self) -> Vec2;
}

// Enums
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Target {
    Food(usize),
    Creature(usize),
    Position(Vec2),
}

impl Target {
    pub fn position(&self, world: &World) -> Option<Vec2> {
        match self {
            Target::Creature(id) => world.creatures.get(id).map(|c| c.position),
            Target::Food(id) => world.food_sources.get(id).map(|f| f.position),
            Target::Position(pos) => Some(*pos),
        }
    }
}

// Structs

pub struct World {
    pub next_id: usize, // every entity has an ID, the ID space is shared
    pub creatures: HashMap<usize, Creature>, // all creatures
    pub food_sources: HashMap<usize, FoodSource>, // all food sources
    pub params: Params, // simulation params
    pub bounds: Bounds, // world boundaries
}

impl World {
    pub fn new(
        creatures: Vec<Creature>,
        food_sources: Vec<FoodSource>,
        params: Params,
        bounds: Bounds,
    ) -> Self {
        let mut world = World {
            next_id: 0,
            creatures: HashMap::new(),
            food_sources: HashMap::new(),
            params,
            bounds,
        };

        for creature in creatures {
            world.add_creature(creature);
        }

        for food_source in food_sources {
            world.add_food_source(food_source);
        }

        world
    }
    pub fn add_creature(&mut self, creature: Creature) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.creatures.insert(id, creature);
        id
    }

    pub fn add_food_source(&mut self, food_source: FoodSource) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.food_sources.insert(id, food_source);
        id
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoodSource {
    /*
     * A FoodSource is a (currently omnivorous) place where creatures flock to
     * if they are hungry. Eating is not currently implemented, but in theory
     * when creatures eat, they subtract amounts from the food source and the
     * food source amount updates.
     */
    pub position: Vec2,
    pub max_amount: f32,
    pub amount: f32,
}

impl FoodSource {
    pub fn new_rand(rng: &mut ThreadRng, bounds: &Bounds) -> Self {
        let max_amount = rng.random_range(50.0..100.0);
        Self {
            position: rvec2_range(rng, bounds),
            max_amount,
            amount: max_amount,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Creature {
    pub position: Vec2, // position in worldspace
    pub velocity: Vec2, // velocity in px space
    pub facing: f32,    // facing angle in radians
    pub strength: f32,
    pub dexterity: f32,
    pub hunger: f32,
    pub hunger_threshold: f32,
    pub hunger_rate: f32,
    // TODO: Add different shapes for evolutionary stuff, create a "size" for
    // visual clarity on how beeg the creature is
    // Also need to add HP for combat
    pub color: Color,
    pub movement_target: Option<Target>,
}

impl Creature {
    fn is_hungry(&self) -> bool {
        self.hunger <= self.hunger_threshold
    }

    fn distance_to_food(&self, food: &FoodSource) -> f32 {
        self.position.distance(food.position)
    }

    fn square_speed(&self) -> f32 {
        self.velocity.length_squared()
    }

    fn max_speed(&self) -> f32 {
        self.dexterity * 10.
    }

    fn acceleration(&self) -> f32 {
        self.dexterity
    }

    fn update_facing(&mut self) {
        let v = self.velocity.normalize_or_zero();
        self.facing = match v {
            Vec2::ZERO => self.facing,
            _ => v.y.atan2(v.x),
        }
    }

    fn move_to_target(&mut self, world: &World) {
        // Move towards the movement_target
        // NOTE: Setting '5' as the threshold for "close enough"
        let to_target: Vec2;
        let squared_distance: f32;
        // First unwrap: if Target not None
        if let Some(target) = self.movement_target {
            // Second unwrap: if Target ID still exists in the world
            if let Some(target_pos) = target.position(world) {
                to_target = target_pos - self.position;
                squared_distance = to_target.length_squared();
            } else {
                // No target, should actually remove the target here?
                return;
            }
        } else {
            // No target
            return;
        }
        if squared_distance < 25. {
            // We made it, zero the velocity
            self.velocity = Vec2::ZERO;
            self.movement_target = None;
            return;
        }
        // Calculate desired speed based on how close we are
        let acceleration_radius = 525.;
        let t = (squared_distance / acceleration_radius).clamp(0.0, 1.0);
        let speed_factor = t * (3.0 - 2.0 * t);
        let desired_speed = self.max_speed() * speed_factor;

        let desired_velocity = to_target.normalize() * desired_speed;

        let steering = desired_velocity - self.velocity;
        let steering_clamped = steering.clamp_length_max(self.acceleration());

        self.velocity += steering_clamped;
        self.velocity = self.velocity.clamp_length_max(self.max_speed());

        self.position += self.velocity * world.params.timestep;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub window_width: f32,
    pub window_height: f32,
    pub padding: f32,
    pub timestep: f32,
    pub damping: f32,
}

impl Default for Params {
    fn default() -> Params {
        Params {
            window_width: 600.,
            window_height: 400.,
            padding: 20.,
            timestep: 1e-2,
            damping: 0.9,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bounds {
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

// Numerics (mainly scaling vectors)
pub fn range_scale(v: f32, old_lo: f32, old_hi: f32, new_lo: f32, new_hi: f32) -> f32 {
    // Scale a value 'v' from [old_lo, old_hi] to [new_lo, new_hi]
    new_lo + v * (new_hi - new_lo) / (old_hi - old_lo)
}

pub fn rvec2_range(rng: &mut ThreadRng, bounds: &Bounds) -> Vec2 {
    // Generate a random Vec2 in the the min/max range of Bounds
    vec2(
        rng.random_range(bounds.x_min..bounds.x_max),
        rng.random_range(bounds.y_min..bounds.y_max),
    )
}

// Random generation
pub fn random_creature(rng: &mut ThreadRng, bounds: &Bounds) -> Creature {
    // TODO: different color for each species
    let _colors = [WHITE, BLUE, BROWN, GOLD, RED];
    let position = rvec2_range(rng, bounds);
    Creature {
        position,
        velocity: vec2(0., 0.),
        facing: 0.,
        strength: 1.,
        dexterity: 1.,
        hunger: 100.,
        hunger_rate: rng.random_range(0.01..0.1),
        hunger_threshold: rng.random_range(25.0..75.0),
        color: _colors[rng.random_range(0.._colors.len())],
        movement_target: None,
    }
}

// Game state updates
fn update_hunger(creature: &mut Creature) {
    // Reduce hunger level based on speed
    creature.hunger -= 0.01 + creature.hunger_rate * creature.square_speed();
    creature.hunger = creature.hunger.clamp(0., 100.);
}

fn find_food(creature: &mut Creature, world: &World) {
    // Move towards closest food source if hungry
    let mut nearest_food: Option<(usize, &FoodSource)> = None;
    let mut distance = f32::MAX;
    for (id, food) in &world.food_sources {
        let food_dist = creature.distance_to_food(&food);
        if food_dist < distance {
            distance = food_dist;
            nearest_food = Some((*id, &food));
        }
    }

    if let Some((food_id, _)) = nearest_food {
        creature.movement_target = Some(Target::Food(food_id));
    }
}

fn find_random_walk_target(rng: &mut ThreadRng, creature: &mut Creature, world: &World) {
    // Set a target in a cone somewhere in front of the creature if we don't
    // have a target already
    if creature.movement_target.is_some() {
        return;
    }
    let distance = rng.random_range(10.0..80.0);
    let angle = rng.random_range(-PI / 6.0..PI / 6.0) + creature.facing;

    // Set a point somewhere in front of the creature as the target, using its
    // facing to determine the offset
    let dx = distance * angle.cos();
    let dy = distance * angle.sin();
    // TODO: Check if in world bounds (just write a function that caps any Vec2
    // to the stored world bounds)
    let target_pos = creature.position + Vec2::new(dx, dy);

    creature.movement_target = Some(Target::Position(target_pos));
    println!("New movemment target is {:?}", creature.movement_target);
}

fn apply_bc(creature: &mut Creature, world: &World) {
    // For now, just repel creatures from the border
    let mut force = Vec2::ZERO;
    let params = world.params;
    let bounds = world.bounds;
    // Force strength is just the distnace to the edge
    if creature.position.x < bounds.x_min + params.padding {
        force.x += params.padding - (creature.position.x - bounds.x_min).max(1.0);
    } else if creature.position.x > bounds.x_max - params.padding {
        force.x -= params.padding - (bounds.x_max - creature.position.x).max(1.0);
    }

    if creature.position.y < bounds.y_min + params.padding {
        force.y += params.padding - (creature.position.y - bounds.y_min).max(1.0);
    } else if creature.position.y > bounds.y_max - params.padding {
        force.y -= params.padding - (bounds.y_max - creature.position.y).max(1.0);
    }

    creature.velocity += force * params.timestep * params.damping;
    creature.position += creature.velocity * params.timestep;
}

pub fn update_world(rng: &mut ThreadRng, world: &mut World) {
    update_food_sources(rng, world);
    update_creatures(rng, world);
}

fn update_food_sources(rng: &mut ThreadRng, world: &mut World) {
    for (_id, food) in world.food_sources.iter_mut() {
        // We can regrow food later, for now we are not doing anything
    }
}

fn update_creatures(rng: &mut ThreadRng, world: &mut World) {
    // Collect all creature IDs, then create new creatures (re-inserting into
    // the hashmap); only works because structs are simple
    let creature_ids: Vec<usize> = world.creatures.keys().cloned().collect();

    for id in creature_ids {
        let mut creature = world.creatures[&id];
        update_hunger(&mut creature);
        if creature.is_hungry() {
            find_food(&mut creature, world);
        } else {
            find_random_walk_target(rng, &mut creature, world);
        }
        creature.move_to_target(world);
        apply_bc(&mut creature, world);
        creature.update_facing();
        world.creatures.insert(id, creature); // Replace the old creature
    }

    // The mutable version the borrow checker hates:
    // for (_id, creature) in world.creatures.iter_mut() {
    //     update_hunger(creature);
    //     if creature.is_hungry() {
    //         // println!("Creature {:?} is finding food", creature);
    //         find_food(creature, world);
    //     } else {
    //         // Target is some random point kinda in front of the creature
    //         // to make it random walk
    //         // println!("Creature {:?} is random walking", creature);
    //         find_random_walk_target(rng, creature, world);
    //     }
    //     creature.move_to_target(world);
    //     apply_bc(creature, world);
    //     creature.update_facing();
    // }
}

/*
* TODO:
* -  Need to change the movement_target to be a generic object so I can change the
* behavior of the creature depending on if it's moving towards a food source
* or another creature
* - Add eating behaviour that replenishes hunger and depletes food source
* - Add a reasonable time step for control
*/
