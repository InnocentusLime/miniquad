in vec3 pos;
in vec4 color;
in vec2 uv;

out vec4 v_color;
out vec2 v_uv;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(pos, 1.0);
    v_color = color;
    v_uv = uv;
}