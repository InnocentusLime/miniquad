uniform mat4 mvp;

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec4 v_color;
layout(location = 2) in vec2 v_uv;

out vec4 f_color;
out vec2 f_uv;

void main() {
    f_color = v_color;
    f_uv = v_uv;
    
    gl_Position = mvp * vec4(v_pos, 1.0);
}