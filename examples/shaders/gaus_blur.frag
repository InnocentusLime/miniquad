uniform sampler2D tex;
uniform vec2 resolution;

in vec2 f_uv;

layout(location = 0) out vec4 o_color;

vec4 blur5(sampler2D image, vec2 uv, vec2 resolution, vec2 direction);

void main() {
    o_color = blur5(tex, f_uv, resolution, vec2(3.0));
}

// Source: https://github.com/Jam3/glsl-fast-gaussian-blur/blob/master/5.glsl
vec4 blur5(sampler2D image, vec2 uv, vec2 resolution, vec2 direction) {
    vec4 color = vec4(0.0);
    vec2 off1 = vec2(1.3333333333333333) * direction;
    color += texture(image, uv) * 0.29411764705882354;
    color += texture(image, uv + (off1 / resolution)) * 0.35294117647058826;
    color += texture(image, uv - (off1 / resolution)) * 0.35294117647058826;
    return color;
}