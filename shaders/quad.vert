#version 450

const vec2[6] POSITIONS = {
    vec2(-1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(1.0, 1.0),
};

const vec2[6] TEX_COORDS = {
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, 0.0),
};

layout(location = 0) out vec2 f_tex_coords;

void main() {
    gl_Position = vec4(POSITIONS[gl_VertexIndex], 0.0, 1.0);
    f_tex_coords = TEX_COORDS[gl_VertexIndex];
}