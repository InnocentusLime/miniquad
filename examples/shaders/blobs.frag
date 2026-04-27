uniform float time;
uniform int blobs_count;
uniform vec2 blobs_positions[32];

in vec2 f_uv;

out vec4 o_color;

float k = 20.0;

float circle(float r, vec3 col, vec2 offset);
vec3 gradient(float shade);

void main() {
    float field = 0.0;
    for (int i = 0; i < 32; i++) {
        // workaround for WebGL: Loop index cannot be compared with non-constant expression
        if (i >= blobs_count) { break; } 
        field += circle(0.03, vec3(0.7, 0.2, 0.8), blobs_positions[i]);
    }
    
    float shade = clamp(field/256.0, 0.0, 1.0);
    
    o_color = vec4(gradient(shade), 1.0);
}
    
float circle(float r, vec3 col, vec2 offset) {
    float d = distance(f_uv, offset);
    return (k * r) / (d * d);
}
    
vec3 band(float shade, float low, float high, vec3 col1, vec3 col2) {
    if ((shade >= low) && (shade <= high)) {
        float delta = (shade - low) / (high - low);
        vec3 col_diff = col2 - col1;
        return col1 + (delta * col_diff);
    } else {
        return vec3(0.0);
    }
}

vec3 gradient(float shade) {
    float time_half = time / 2.0;
    vec2 sin_cos = vec2(sin(time_half), cos(time_half)) * 0.25 + 0.25;
    vec3 color = vec3(sin_cos.x, 0.0, sin_cos.y);
    
    vec3 col1 = vec3(0.01 , 0.0  , 0.99 );
    vec3 col2 = vec3(0.99 , 0.0  , 0.01 );
    vec3 col3 = vec3(0.02 , 0.98 , 0.02 );
    vec3 col4 = vec3(0.015, 0.015, 0.985);
    vec3 col5 = vec3(0.02 , 0.02 , 0.02 );
    
    color += band(shade, 0.0, 0.3, color, col1 );
    color += band(shade, 0.3, 0.6, col1, col2 );
    color += band(shade, 0.6, 0.8, col2, col3 );
    color += band(shade, 0.8, 0.9, col3, col4 );
    color += band(shade, 0.9, 1.0, col4, col5 );
    
    return color;
}