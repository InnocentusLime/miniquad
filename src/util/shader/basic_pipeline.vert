uniform mat4 view_projection;

layout(location = 0) in vec2 v_pos;
layout(location = 1) in vec4 v_color;

out vec4 f_color;

void main() {
    f_color = v_color;
    
    gl_Position = view_projection * vec4(v_pos, 0, 1);
}