// uniform sampler2D texture1;
//
// in vec2 uv;
// out vec4 c;
//
// void main() { 
// 	vec4 s = texture(texture1, uv);
//     c = vec4(vec3(1.0 - s), 1.0);
// }

uniform sampler2D framebuffer;
uniform sampler2D dudv;
uniform float time;

in vec2 uv;
out vec4 c;

vec2 random2(vec2 st){
    st = vec2( dot(st,vec2(127.1,311.7)),
              dot(st,vec2(269.5,183.3)) );
    return -1.0 + 2.0*fract(sin(st)*43758.5453123);
}

// https://thebookofshaders.com/edit.php#11/2d-gnoise.frag
float perlin(vec2 st) {
    vec2 i = floor(st);
    vec2 f = fract(st);
    vec2 u = f*f*(3.0-2.0*f);
    return mix( mix( dot( random2(i + vec2(0.0,0.0) ), f - vec2(0.0,0.0) ),
                     dot( random2(i + vec2(1.0,0.0) ), f - vec2(1.0,0.0) ), u.x),
                mix( dot( random2(i + vec2(0.0,1.0) ), f - vec2(0.0,1.0) ),
                     dot( random2(i + vec2(1.0,1.0) ), f - vec2(1.0,1.0) ), u.x), u.y);
}

float multisampled_perlin(vec2 offset) {
	vec2 st = uv + offset;
	float height = perlin(st * 10.0) * 0.5;
	height += perlin(st * 20.0) * 0.3;
	height += perlin(st * 30.0) * 0.2;
	return height;
}

// use perlin to generate dudv and normal map
void main() {
	// float specular_strength = 0.75;
	// vec3 light_color = vec3(1.0);
	// vec3 color = vec3(1.0, 0.5, 0.2);
	//
	// vec3 frag_normal = vec3(
	// 			multisampled_perlin(vec2(-1, -1) * time * 0.015),
	// 			multisampled_perlin(vec2(1, -1) * time * 0.01),
	// 			multisampled_perlin(vec2(-1, 1) * time * 0.012)
	// 		);
	//
	// vec3 camera_position = vec3(0, 0, -1);
	// vec3 light_source = vec3(-1, -1, -3);
	// vec3 frag_position = vec3(uv, 0);
	//
	// vec3 norm = normalize(frag_normal);
	// vec3 light_dir = normalize(light_source - frag_position);  
	//
	// vec3 view_dir = normalize(camera_position - frag_position);
	// vec3 reflect_dir = reflect(-light_dir, norm); 
	// float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
	// vec3 specular = specular_strength * spec * light_color;  

	vec2 uv_offset = texture(dudv, time * 0.09 + uv).rg;
	uv_offset = (uv_offset - 0.5) * 2.0;

	float s1 = multisampled_perlin(vec2(-1, 1) * time * 0.1 + uv_offset * 0.002) * 0.5 + 0.25;
	float s2 = multisampled_perlin(vec2(-1, -1) * time * 0.15) * 0.5 + 0.25;
	float sample = s1 + s2;
	// float s3 = multisampled_perlin(vec2(-1, 0) * time * 0.08 + uv_offset * 0.01) * 0.5 + 0.15;
	// float sample = s1 + s2 + s3;

	float cutoff = 0.5;
	float sl = step(cutoff, sample);
	float l = smoothstep(cutoff, 1.0, sample);
	vec3 specular = vec3(l, l, l);

	vec3 col = vec3(texture(framebuffer, uv + uv_offset * 0.001));
	c = vec4(col + specular, 1.0);
}  
