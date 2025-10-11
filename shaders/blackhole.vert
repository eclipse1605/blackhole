#version 330 core

layout(location = 0) in vec3 aPos;

out vec2 vTexCoord;

void main() {
    gl_Position = vec4(aPos, 1.0);
    // convert from [-1, 1] to [0, 1] for texture coordinates
    vTexCoord = aPos.xy * 0.5 + 0.5;
}
