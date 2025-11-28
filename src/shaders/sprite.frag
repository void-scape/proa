uniform sampler2D texture1;

in vec2 uv;
out vec4 c;

void main() {
    c = texture(texture1, uv);
} 
