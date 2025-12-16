in vec2 pos;
in vec2 uv;

out vec2 texcoord;

void main() {
    gl_Position = vec4(pos, 0, 1);
    texcoord = uv;
}