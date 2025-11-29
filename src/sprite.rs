use crate::{
    shader::uniform,
    texture::{self, Image},
};
use glam::{Mat4, Quat, Vec2, Vec3};
use glazer::glow::{self, HasContext};

#[derive(Clone, Copy)]
pub struct Sprite {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec2,
    pub image: Image,
}

impl Sprite {
    pub fn from_image(gl: &glow::Context, path: &str) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::default(),
            scale: Vec2::ONE,
            image: texture::load_image(gl, path),
        }
    }

    pub fn from_size(gl: &glow::Context, size: Vec2) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::default(),
            scale: size,
            image: texture::default_image(gl),
        }
    }
}

pub struct SpriteRenderer {
    shader: glow::Program,
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
    _ebo: glow::Buffer,
}

impl SpriteRenderer {
    pub fn new(gl: &glow::Context, width: usize, height: usize) -> Self {
        #[rustfmt::skip]
        let vertices = [
             // positions       texture coords
             0.5,  0.5, 0.0,    1.0, 1.0, // top right
             0.5, -0.5, 0.0,    1.0, 0.0, // bottom right
            -0.5, -0.5, 0.0,    0.0, 0.0, // bottom let
            -0.5,  0.5, 0.0,    0.0, 1f32 // top let 
        ];
        #[rustfmt::skip]
        let indices = [
            0, 1, 3,
            1, 2, 3u32,
        ];

        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();
            let ebo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let data =
                core::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * 4);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            let data =
                core::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4);
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, data, glow::STATIC_DRAW);

            let stride = 5 * 4;
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 3 * 4);
            gl.enable_vertex_attrib_array(1);

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            let shader = crate::compile_shader!(gl, "shaders/sprite.vert", "shaders/sprite.frag");
            gl.use_program(Some(shader));

            uniform(gl, shader, "proj_matrix", |location| {
                let w_2 = width as f32 / 2.0;
                let h_2 = height as f32 / 2.0;
                let proj_matrix = Mat4::orthographic_rh_gl(-w_2, w_2, -h_2, h_2, -1000.0, 1000.0);
                gl.uniform_matrix_4_f32_slice(location, false, &proj_matrix.to_cols_array());
            });

            Self {
                shader,
                vao,
                _vbo: vbo,
                _ebo: ebo,
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

    #[allow(unused)]
    pub fn render(&self, gl: &glow::Context, sprite: &Sprite) {
        unsafe {
            gl.use_program(Some(self.shader));
            gl.bind_texture(glow::TEXTURE_2D, Some(sprite.image.texture));

            uniform(gl, self.shader, "model_matrix", |location| {
                let model_matrix = Mat4::from_scale_rotation_translation(
                    (sprite.scale * Vec2::new(sprite.image.width, sprite.image.height)).extend(1.0),
                    sprite.rotation,
                    sprite.translation,
                );
                gl.uniform_matrix_4_f32_slice(location, false, &model_matrix.to_cols_array());
            });

            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
        }
    }
}
