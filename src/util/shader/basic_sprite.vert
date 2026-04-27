uniform mat4 view_projection;
uniform vec2 width_height;

layout(location=0) in vec2 v_pos;
layout(location=1) in vec2 v_uv_not_normalized;

out vec2 f_uv;

void main() {
    vec2 uv = v_uv_not_normalized / width_height;
    uv.y = 1.0 - uv.y;
    
    f_uv = uv;
    
    gl_Position = view_projection * vec4(v_pos, 0, 1);
}