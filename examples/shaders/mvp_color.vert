uniform mat4 mvp;

layout(location = 0) in vec3 v_pos;
layout(location = 1) in vec4 v_color;

out vec4 f_color;

void main() {
    f_color = v_color;
    
    gl_Position = mvp * vec4(v_pos, 1.0);
}