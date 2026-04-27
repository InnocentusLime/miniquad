uniform sampler2D tex;

in vec2 f_uv;

layout(location = 0) out vec4 o_color;

void main() {
    o_color = texture(tex, f_uv);
}