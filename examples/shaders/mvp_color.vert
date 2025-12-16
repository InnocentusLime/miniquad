in vec3 pos;
in vec4 color;

out vec4 v_color;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(pos, 1.0);
    v_color = color;
}