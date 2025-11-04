#version 330 core

in vec2 vTexCoord;
out vec4 FragColor;

uniform sampler2D colorMap;

void main() {
    // Simple, cheap fallback: sample the provided color map for a pleasing placeholder
    vec3 col = texture(colorMap, fract(vTexCoord)).rgb;
    FragColor = vec4(col, 1.0);
}
