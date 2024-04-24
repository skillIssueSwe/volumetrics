#version 120

vec4 positions[3] = vec4[](
    vec4(0.0, -0.5,0.0,1.0),
    vec4(0.5, 0.5,0.0,1.0),
    vec4(-0.5, 0.5,0.0,1.0)
);

void main() {
    gl_pointSize = 2.0;
    gl_position = vec4(0.0,0.0,0.0,1.0);
}
