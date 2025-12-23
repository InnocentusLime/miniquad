uniform mat4 view_projection;
uniform vec2 width_height;

layout(location=0) in vec2 pos;
layout(location=1) in vec2 texcoord;
layout(location=2) in vec4 color;

out vec4 v_color;
out vec2 v_texcoord;

void main() {
    gl_Position = view_projection * vec4(pos, 0, 1);
    v_color = color;
    v_texcoord = texcoord;
}