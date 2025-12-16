in vec2 pos;
in uvec4 color;

out vec4 v_color;

void main() {
    gl_Position = vec4(pos, 0, 1);
    v_color = vec4(color) / 255.0;
}