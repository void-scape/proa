//! Inspired by the lovely chain simulation I stumbled into:
//!
//! https://github.com/argonautcode/animal-proc-anim
//!
//! ## Things I want to do
//! - Sylized water with specular reflections
//! - Koi like color with uniques patterns for each fish
//! - Various fish sizes
//! - Boid flocking

use crate::{
    boids::BoidMemory,
    pebbles::PebbleRenderer,
    postprocess::PostProcessor,
    shader::uniform,
    sprite::{Sprite, SpriteRenderer},
};
use glam::{Mat4, Quat, Vec2, Vec3};
use glazer::winit::{self, event::WindowEvent};
use glow::HasContext;

mod boids;
mod pebbles;
mod postprocess;
mod rng;
mod shader;
mod sprite;
mod texture;

#[derive(Default)]
pub struct Memory {
    world: Option<World>,
}

const SEGMENTS: usize = 14;
struct World {
    cursor: Vec2,
    joints: Vec<[Joint; SEGMENTS]>,
    joint_renderer: JointRenderer,
    sprites: [Sprite; SEGMENTS],
    sprite_renderer: SpriteRenderer,
    pebble_renderer: PebbleRenderer,
    boid_memory: BoidMemory,
    postprocessor: PostProcessor,
}

#[unsafe(no_mangle)]
pub fn handle_input(
    glazer::PlatformInput {
        input, memory, gl, ..
    }: glazer::PlatformInput<Memory>,
) {
    match input {
        WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    physical_key:
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                    ..
                },
            ..
        } => {
            std::process::exit(0);
        }
        WindowEvent::CursorMoved { position, .. } => {
            if let Some(world) = &mut memory.world {
                world.cursor = Vec2::new(position.x as f32, position.y as f32);
            }
        }
        WindowEvent::Resized(size) => {
            if let Some(world) = &mut memory.world {
                let w = size.width as usize;
                let h = size.height as usize;
                // NOTE: Fish would look to small if they were resized
                // world.joint_renderer.resize(gl, w, h);
                // world.sprite_renderer.resize(gl, w, h);
                world.postprocessor.resize(gl, w, h);
            }
        }
        _ => {}
    }
}

#[unsafe(no_mangle)]
pub fn update_and_render(
    glazer::PlatformUpdate {
        memory,
        gl,
        delta,
        width,
        height,
        ..
    }: glazer::PlatformUpdate<Memory>,
) {
    let speed = 350.0;
    let separation = 20.0;
    let min_joint_angle = std::f32::consts::PI / 2.0;

    let joint_sizes = [
        10.0, 18.0, 25.0, 23.0, 24.0, 23.0, 22.0, 21.0, 16.0, 14.0, 10.0, 6.0, 3.0, 2.0,
    ];

    let world = memory.world.get_or_insert_with(|| World {
        cursor: Vec2::ZERO,
        joints: Vec::new(),
        joint_renderer: JointRenderer::new(gl, width, height),
        sprites: [Sprite::from_size(gl, Vec2::splat(16.0)); SEGMENTS],
        sprite_renderer: SpriteRenderer::new(gl, width, height),
        pebble_renderer: PebbleRenderer::new(gl),
        boid_memory: BoidMemory::default(),
        postprocessor: PostProcessor::new(gl, width, height),
    });

    world.boid_memory.update(delta);
    let num_boids = world.boid_memory.boids().len();
    if world.joints.len() < num_boids {
        while world.joints.len() < num_boids {
            let mut offset = 0.0;
            world.joints.push(joint_sizes.map(|size| {
                let translation = Vec2::X * offset;
                offset += separation;
                Joint { size, translation }
            }))
        }
    }

    for (boid, joints) in world
        .boid_memory
        .boids()
        .iter()
        .zip(world.joints.iter_mut())
    {
        let head_translation = boid.translation;
        let mut target =
            head_translation + boid.velocity.normalize_or(Vec2::X) * separation + speed * delta;

        for i in 0..joints.len() {
            let joint = &mut joints[i];
            let offset = target - joint.translation;
            let current_distance = offset.length();

            if current_distance > 0.0 {
                let constrained_position = target - offset.normalize() * separation;
                joint.translation = constrained_position;
            }

            target = joint.translation;
            if i < joints.len() - 2 {
                let joint_translation = joints[i].translation;
                let anchor_translation = joints[i + 1].translation;
                let joint2_translation = joints[i + 2].translation;

                let normalized_joint =
                    (joint_translation - anchor_translation).normalize_or(Vec2::X);
                let normalized_joint2 =
                    (joint2_translation - anchor_translation).normalize_or(Vec2::X);
                let angle = normalized_joint.angle_to(normalized_joint2);

                if angle.abs() < min_joint_angle {
                    let rotation = min_joint_angle * angle.signum();
                    let constrained_direction = normalized_joint.rotate(Vec2::from_angle(rotation));
                    let joint2 = &mut joints[i + 2];
                    joint2.translation = constrained_direction * separation + anchor_translation;
                }
            }
        }

        for (sprite, joint) in world.sprites.iter_mut().zip(joints.iter()) {
            sprite.translation = joint.translation.extend(10.0);
        }
    }

    unsafe {
        world.postprocessor.bind_framebuffer(gl);
        gl.clear_color(0.0, 0.1, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        gl.enable(glow::DEPTH_TEST);
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        world.pebble_renderer.render(gl);

        for joints in world.joints.iter() {
            let vertex_count = world.joint_renderer.bind_ellipse(gl);
            let mut render_pectoral_fins = |seg: usize, size: f32| {
                let joint = joints[seg];
                let heading = (joints[seg - 1].translation - joint.translation).normalize_or_zero();
                for side in [Vec2::Y, Vec2::NEG_Y].into_iter() {
                    let transform = Mat4::from_scale_rotation_translation(
                        Vec3::ONE * size,
                        Quat::from_rotation_z(
                            side.rotate(heading).to_angle() - 0.85 * side.y.signum(),
                        ),
                        (joint.translation + side.rotate(heading) * 20.0).extend(-1.0),
                    );
                    world
                        .joint_renderer
                        .render(gl, transform, vertex_count, glow::TRIANGLE_FAN);
                }
            };
            render_pectoral_fins(3, 0.8);
            render_pectoral_fins(7, 0.95);

            // caudal fin
            let seg = joints.len() - 1;
            let joint = joints[seg];
            let heading = (joints[seg - 1].translation - joint.translation).normalize_or_zero();
            let transform = Mat4::from_scale_rotation_translation(
                Vec3::new(0.3, 1.2, 1.0),
                Quat::from_rotation_z(heading.to_angle() + std::f32::consts::PI / 2.0),
                joint.translation.extend(-1.0),
            );
            world
                .joint_renderer
                .render(gl, transform, vertex_count, glow::TRIANGLE_FAN);

            let vertex_count = world.joint_renderer.bind_joints(gl, joints, delta);
            world
                .joint_renderer
                .render(gl, Mat4::IDENTITY, vertex_count, glow::TRIANGLE_STRIP);
        }

        // debug spine
        // for sprite in world.sprites.iter() {
        //     world.sprite_renderer.render(gl, sprite);
        // }

        // post processing
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        // gl.clear_color(0.1, 0.1, 0.1, 1.0);
        // gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        gl.disable(glow::DEPTH_TEST);
        world.postprocessor.render_to_active_framebuffer(gl, delta);
    }
}

#[derive(Clone, Copy)]
struct Joint {
    size: f32,
    translation: Vec2,
}

pub struct JointRenderer {
    shader: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    vbo_len: usize,
    time: f32,
}

impl JointRenderer {
    pub fn new(gl: &glow::Context, width: usize, height: usize) -> Self {
        unsafe {
            let shader = crate::compile_shader!(gl, "shaders/joint.vert", "shaders/joint.frag");
            gl.use_program(Some(shader));

            uniform(gl, shader, "proj_matrix", |location| {
                let w_2 = width as f32 / 2.0;
                let h_2 = height as f32 / 2.0;
                let proj_matrix = Mat4::orthographic_rh_gl(-w_2, w_2, -h_2, h_2, -1000.0, 1000.0);
                gl.uniform_matrix_4_f32_slice(location, false, &proj_matrix.to_cols_array());
            });

            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * 4, 0);
            gl.enable_vertex_attrib_array(0);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Self {
                shader,
                vao,
                vbo,
                vbo_len: 0,
                time: 0.0,
            }
        }
    }

    pub fn resize(&self, gl: &glow::Context, width: usize, height: usize) {
        unsafe {
            gl.use_program(Some(self.shader));
            uniform(gl, self.shader, "proj_matrix", |location| {
                let w_2 = width as f32 / 2.0;
                let h_2 = height as f32 / 2.0;
                let proj_matrix = Mat4::orthographic_rh_gl(-w_2, w_2, -h_2, h_2, -1000.0, 1000.0);
                gl.uniform_matrix_4_f32_slice(location, false, &proj_matrix.to_cols_array());
            });
        }
    }

    fn bind_joints(&mut self, gl: &glow::Context, joints: &[Joint], dt: f32) -> usize {
        unsafe {
            self.time += dt;

            gl.use_program(Some(self.shader));
            uniform(gl, self.shader, "ripple", |location| {
                gl.uniform_1_f32(location, 0.0);
            });

            let mut vertices = Vec::with_capacity(joints.len() * 2);
            let mut last_heading = Vec2::X;
            for i in (0..joints.len()).rev() {
                if i > 0 {
                    let joint = &joints[i];
                    let next = &joints[i - 1];
                    let heading = (next.translation - joint.translation).normalize_or(Vec2::X);
                    let vert = Vec2::Y.rotate(heading) * joint.size + joint.translation;
                    vertices.push(vert.extend(0.0));
                    let vert = Vec2::NEG_Y.rotate(heading) * joint.size + joint.translation;
                    vertices.push(vert.extend(0.0));
                    last_heading = heading;
                } else {
                    let joint = &joints[i];
                    let vert = Vec2::Y.rotate(last_heading) * joint.size + joint.translation;
                    vertices.push(vert.extend(0.0));
                    let vert = Vec2::NEG_Y.rotate(last_heading) * joint.size + joint.translation;
                    vertices.push(vert.extend(0.0));
                }
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            let data = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<Vec3>(),
            );
            if self.vbo_len <= vertices.len() {
                self.vbo_len = vertices.len();
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::DYNAMIC_DRAW);
            } else {
                gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, data);
            }

            vertices.len()
        }
    }

    fn bind_ellipse(&mut self, gl: &glow::Context) -> usize {
        unsafe {
            gl.use_program(Some(self.shader));
            uniform(gl, self.shader, "ripple", |location| {
                gl.uniform_1_f32(location, 1.0);
            });
            uniform(gl, self.shader, "time", |location| {
                gl.uniform_1_f32(location, self.time);
            });

            let xradius = 15.0;
            let yradius = 25.0;

            let segments = 20;
            let mut vertices = Vec::with_capacity(segments * 2);
            for i in 0..segments {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                vertices.push(Vec3::new(xradius * angle.cos(), yradius * angle.sin(), 1.0));
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            let data = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<Vec3>(),
            );
            if self.vbo_len <= vertices.len() {
                self.vbo_len = vertices.len();
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::DYNAMIC_DRAW);
            } else {
                gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, data);
            }

            vertices.len()
        }
    }

    fn render(&mut self, gl: &glow::Context, model_matrix: Mat4, vertices: usize, mode: u32) {
        unsafe {
            gl.use_program(Some(self.shader));
            gl.bind_vertex_array(Some(self.vao));

            uniform(gl, self.shader, "model_matrix", |location| {
                gl.uniform_matrix_4_f32_slice(location, false, &model_matrix.to_cols_array());
            });

            #[cfg(not(target_arch = "wasm32"))]
            {
                // outlines
                gl.line_width(4.0);
                uniform(gl, self.shader, "color", |location| {
                    gl.uniform_4_f32(location, 1.0, 1.0, 1.0, 1.0);
                });
                gl.polygon_mode(glow::FRONT_AND_BACK, glow::LINE);
                gl.draw_arrays(mode, 0, vertices as i32);
            }

            // fill
            gl.disable(glow::DEPTH_TEST);
            uniform(gl, self.shader, "color", |location| {
                gl.uniform_4_f32(location, 1.0, 0.0, 0.0, 1.0);
            });
            gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
            gl.draw_arrays(mode, 0, vertices as i32);
            gl.enable(glow::DEPTH_TEST);
        }
    }
}
