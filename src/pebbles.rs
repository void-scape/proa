use crate::{compile_shader, postprocess, texture};
use glow::HasContext;

pub struct PebbleRenderer {
    pebbles: glow::Texture,
    shader: glow::Program,
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
}

impl PebbleRenderer {
    pub fn new(gl: &glow::Context) -> Self {
        let (vao, vbo) = postprocess::fullscreen_quad(gl);
        Self {
            pebbles: texture::load_texture(gl, "assets/pebbles.jpg"),
            shader: compile_shader!(gl, "shaders/postprocess.vert", "shaders/pebbles.frag"),
            vao,
            _vbo: vbo,
        }
    }

    pub fn render(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.shader));
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_texture(glow::TEXTURE_2D, Some(self.pebbles));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}
