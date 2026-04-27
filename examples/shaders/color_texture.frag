uniform sampler2D tex;

in vec4 f_color;
in vec2 f_uv;

layout(location = 0) out vec4 o_color;

void main() {
    o_color = f_color * texture(tex, f_uv);
}