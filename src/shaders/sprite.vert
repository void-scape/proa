layout (location = 0) in vec3 position;
layout (location = 1) in vec2 tex_coord;

uniform mat4 proj_matrix;
uniform mat4 model_matrix;

out vec2 uv;

void main() {
	gl_Position = proj_matrix * model_matrix * vec4(position, 1.0);
	uv = tex_coord;
}
