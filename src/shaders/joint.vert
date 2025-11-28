layout (location = 0) in vec3 position;

uniform mat4 model_matrix;
uniform mat4 proj_matrix;
uniform float ripple;
uniform float time;

void main() {
	float tau = 3.14 * 2.0;
	vec3 translation = model_matrix[3].xyz;
	float ripple_factor = ripple * sin(time * tau + translation.x * 0.001);
	vec3 rippled_position = vec3(0.0, ripple_factor, 0.0) + position;
	gl_Position = proj_matrix * model_matrix * vec4(rippled_position, 1.0);
}
