uniform mat4 view_projection;

in vec2 pos;
in vec4 color;

out vec4 v_color;

void main() {
    gl_Position = view_projection * vec4(pos, 0, 1);
    v_color = color;
}