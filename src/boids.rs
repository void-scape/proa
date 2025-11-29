use crate::rng;
use glam::Vec2;

const BOUNDS: Vec2 = Vec2::new(720.0 / 4.0, 1280.0 / 4.0);
const BOID_COUNT: usize = 8;
const MAX_SPEED: f32 = 150.0;
const MAX_SPEED_SQ: f32 = MAX_SPEED * MAX_SPEED;
const MIN_SPEED: f32 = 100.0;
const MIN_SPEED_SQ: f32 = MIN_SPEED * MIN_SPEED;

pub struct BoidMemory {
    boids: [Boid; BOID_COUNT],
    //
    margin: Vec2,
    turn_factor: f32,
    //
    separation_factor: f32,
    cohesion_factor: f32,
    alignment_factor: f32,
    //
    view_radius_squared: f32,
    separation_radius_squared: f32,
}

impl Default for BoidMemory {
    fn default() -> Self {
        BoidMemory {
            boids: core::array::from_fn(|i| Boid {
                translation: Vec2::new(rng::sample_f32(i * 2), rng::sample_f32(i * 3 + 1))
                    .normalize_or_zero()
                    * 2.0
                    * BOUNDS
                    - BOUNDS,
                velocity: Vec2::new(rng::sample_f32(i * 8), rng::sample_f32(i * 9 + 1))
                    .normalize_or_zero()
                    * 2.0
                    * MAX_SPEED
                    - MAX_SPEED,
            }),
            //
            turn_factor: 1.0,
            margin: BOUNDS / 4.0,
            //
            separation_factor: 0.025,
            cohesion_factor: 0.0005,
            alignment_factor: 0.01,
            //
            view_radius_squared: 24f32.powi(2),
            separation_radius_squared: 12f32.powi(2),
        }
    }
}

impl BoidMemory {
    pub fn update(&mut self, dt: f32) {
        boid_forces(self);
        avoid_bounds(self);
        apply_velocity(self, dt);
    }

    pub fn boids(&self) -> &[Boid] {
        &self.boids
    }
}

pub struct Boid {
    pub translation: Vec2,
    pub velocity: Vec2,
}

fn apply_velocity(memory: &mut BoidMemory, dt: f32) {
    for boid in memory.boids.iter_mut() {
        if boid.velocity.length_squared() > MAX_SPEED_SQ {
            boid.velocity = boid.velocity.normalize_or_zero() * MAX_SPEED;
        } else if boid.velocity.length_squared() < MIN_SPEED_SQ {
            boid.velocity = boid.velocity.normalize_or_zero() * MIN_SPEED;
        }
        boid.translation += boid.velocity * dt;
    }
}

fn avoid_bounds(memory: &mut BoidMemory) {
    for boid in memory.boids.iter_mut() {
        if boid.translation.x < -BOUNDS.x + memory.margin.x {
            boid.velocity.x += memory.turn_factor;
        }
        if boid.translation.x > BOUNDS.x - memory.margin.x {
            boid.velocity.x -= memory.turn_factor;
        }
        if boid.translation.y < -BOUNDS.y + memory.margin.y {
            boid.velocity.y += memory.turn_factor;
        }
        if boid.translation.y > BOUNDS.y - memory.margin.y {
            boid.velocity.y -= memory.turn_factor;
        }
    }
}

fn boid_forces(memory: &mut BoidMemory) {
    let mut velocity_changes = [Vec2::ZERO; BOID_COUNT];

    for (i, current_boid) in memory.boids.iter().enumerate() {
        let mut separation = Vec2::ZERO;
        let mut cohesion_center = Vec2::ZERO;
        let mut alignment_avg = Vec2::ZERO;
        let mut cohesion_count = 0;
        let mut alignment_count = 0;

        for j in 0..memory.boids.len() {
            if i != j {
                let other_boid = &memory.boids[j];
                let distance_sq = current_boid
                    .translation
                    .distance_squared(other_boid.translation);

                if distance_sq <= memory.separation_radius_squared {
                    separation += current_boid.translation - other_boid.translation;
                }

                if distance_sq <= memory.view_radius_squared {
                    cohesion_center += other_boid.translation;
                    alignment_avg += other_boid.velocity;
                    cohesion_count += 1;
                    alignment_count += 1;
                }
            }
        }

        let mut total_change = separation * memory.separation_factor;
        if cohesion_count > 0 {
            cohesion_center /= cohesion_count as f32;
            total_change += (cohesion_center - current_boid.translation) * memory.cohesion_factor;
        }
        if alignment_count > 0 {
            alignment_avg /= alignment_count as f32;
            total_change += (alignment_avg - current_boid.velocity) * memory.alignment_factor;
        }
        velocity_changes[i] = total_change;
    }

    for (i, boid) in memory.boids.iter_mut().enumerate() {
        boid.velocity += velocity_changes[i];
    }
}
