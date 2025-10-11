#version 330 core

in vec2 vTexCoord;
out vec4 FragColor;

uniform vec2 u_resolution;
uniform vec3 u_camera_pos;
uniform mat3 u_view_matrix;
uniform float u_fov;

void main() {
    vec2 uv = (gl_FragCoord.xy / u_resolution.xy) * 2.0 - 1.0;
    uv.x *= u_resolution.x / u_resolution.y;
    
    float tanHalfFov = tan(radians(u_fov * 0.5));
    vec3 dir = normalize(vec3(uv.x * tanHalfFov, -uv.y * tanHalfFov, 1.0));
    
    dir = u_view_matrix * dir;
    
    vec3 color = vec3(0.0);
    
    color.r = abs(uv.x);
    color.g = abs(uv.y);
    
    color.b = dir.z * 0.5 + 0.5;
    
    FragColor = vec4(color, 1.0);
}
