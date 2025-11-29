use glow::HasContext;
use image::EncodableLayout;

#[derive(Clone, Copy)]
pub struct Image {
    pub texture: glow::Texture,
    pub width: f32,
    pub height: f32,
}

pub fn default_image(gl: &glow::Context) -> Image {
    Image {
        texture: load_texture_inner(gl, &[255; 3], 1, 1),
        width: 1.0,
        height: 1.0,
    }
}

pub fn load_image(gl: &glow::Context, path: &str) -> Image {
    let image = image::open(path).unwrap();
    let width = image.width();
    let height = image.height();
    let rgb = image.flipv().to_rgb8();
    let bytes = rgb.as_bytes();
    let texture = load_texture_inner(gl, bytes, width, height);

    Image {
        texture,
        width: width as f32,
        height: height as f32,
    }
}

pub fn load_texture(gl: &glow::Context, path: &str) -> glow::Texture {
    let image = image::open(path).unwrap();
    let width = image.width();
    let height = image.height();
    let rgb = image.flipv().to_rgb8();
    let bytes = rgb.as_bytes();
    load_texture_inner(gl, bytes, width, height)
}

fn load_texture_inner(gl: &glow::Context, bytes: &[u8], width: u32, height: u32) -> glow::Texture {
    unsafe {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
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

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as i32,
            width as i32,
            height as i32,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(bytes)),
        );

        texture
    }
}
