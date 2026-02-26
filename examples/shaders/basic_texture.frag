in vec2 texcoord;

out vec4 f_color;

uniform sampler2D tex;

void main() {
    f_color = texture(tex, texcoord);
}