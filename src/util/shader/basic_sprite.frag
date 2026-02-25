in vec4 v_color;
in vec2 v_texcoord;

out vec4 f_color;

uniform sampler2D tex;
uniform vec2 width_height;

void main() {
    vec2 finuv = v_texcoord / width_height;
    finuv.y = 1.0 - finuv.y;
    f_color = v_color * texture(tex, finuv);
}