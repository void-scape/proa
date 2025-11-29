use crate::{shader::uniform, texture};
use glow::HasContext;

pub struct PostProcessor {
    data: PostProcessorData,
    shader: glow::Program,
    dudv: glow::Texture,
    time: f32,
}

impl PostProcessor {
    pub fn new(gl: &glow::Context, width: usize, height: usize) -> Self {
        let shader = crate::compile_shader!(gl, "shaders/postprocess.vert", "shaders/water.frag");
        Self {
            data: PostProcessorData::new(gl, width, height),
            shader,
            dudv: texture::load_texture(gl, "assets/dudv.jpg"),
            time: 0.0,
        }
    }

    pub fn resize(&mut self, gl: &glow::Context, width: usize, height: usize) {
        self.data.resize(gl, width, height);
    }

    pub fn bind_framebuffer(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.data.framebuffer));
        }
    }

    pub fn render_to_active_framebuffer(&mut self, gl: &glow::Context, dt: f32) {
        self.time += dt;
        unsafe {
            gl.use_program(Some(self.shader));
            gl.bind_vertex_array(Some(self.data.vao));

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.data.texture));

            gl.active_texture(glow::TEXTURE0 + 1);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.dudv));

            uniform(gl, self.shader, "framebuffer", |location| {
                gl.uniform_1_i32(location, 0);
            });
            uniform(gl, self.shader, "dudv", |location| {
                gl.uniform_1_i32(location, 1);
            });
            uniform(gl, self.shader, "time", |location| {
                gl.uniform_1_f32(location, self.time);
            });

            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            gl.active_texture(glow::TEXTURE0);
        }
    }
}

struct PostProcessorData {
    framebuffer: glow::Framebuffer,
    texture: glow::Texture,
    vao: glow::VertexArray,
    _vbo: glow::Buffer,
}

impl PostProcessorData {
    fn new(gl: &glow::Context, width: usize, height: usize) -> Self {
        unsafe {
            let framebuffer = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                width as i32,
                height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );

            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                panic!("postprocessing framebuffer is not complete");
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            let (vao, vbo) = fullscreen_quad(gl);

            Self {
                framebuffer,
                texture,
                vao,
                _vbo: vbo,
            }
        }
    }

    pub fn resize(&mut self, gl: &glow::Context, width: usize, height: usize) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                width as i32,
                height as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(self.texture),
                0,
            );

            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                panic!("postprocessing framebuffer is not complete");
            }
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }
}

pub fn fullscreen_quad(gl: &glow::Context) -> (glow::VertexArray, glow::Buffer) {
    #[rustfmt::skip]
        let vertices: [f32; 24] = [
            // position  // uv
            -1.0,  1.0,  0.0, 1.0,
            -1.0, -1.0,  0.0, 0.0,
             1.0, -1.0,  1.0, 0.0,
             //
            -1.0,  1.0,  0.0, 1.0,
             1.0, -1.0,  1.0, 0.0,
             1.0,  1.0,  1.0, 1.0,
        ];

    unsafe {
        let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        let data = core::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * 4);
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

        let stride = 4 * 4;
        gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 2 * 4);
        gl.enable_vertex_attrib_array(1);

        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        (vao, vbo)
    }
}
