#version 330 core

in vec2 vTexCoord;
out vec4 FragColor;

const float PI = 3.14159265359;
const float EPSILON = 0.001;

// use normalized units where R_S = 1.0
const float R_S = 1.0;
const float D_LAMBDA = 0.1;  // smaller step for normalized units

uniform vec2 u_resolution;
uniform float u_time;
uniform vec3 u_camera_pos;
uniform mat3 u_view_matrix;
uniform float u_fov;
uniform bool u_render_disk;
uniform bool u_gravitational_lensing;

// 🆕 Texture uniforms for Ross Ning–style visuals
uniform sampler2D colorMap;
uniform samplerCube skybox;

float hash(vec3 p) {
    p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

vec3 accel(float h2, vec3 pos) {
    float r2 = dot(pos, pos);
    float r5 = pow(r2, 2.5);
    return -1.5 * h2 * pos / r5;
}

bool crossesDisk(vec3 oldPos, vec3 newPos, float innerR, float outerR) {
    bool crossed = (oldPos.y * newPos.y < 0.0);
    if (!crossed) return false;
    float r = length(newPos.xz);
    return (r >= innerR && r <= outerR);
}

// 🆕 Disk color now uses texture lookup
vec3 getDiskColor(vec3 pos) {
    float r = length(pos.xz);
    float angle = atan(pos.z, pos.x);
    
    // Normalize angle to [0,1] for texture sampling
    float u = fract(angle / (2.0 * PI));
    // Map radius to [0,1] range (inner radius = 2.6Rs, outer = 12Rs)
    float v = clamp((r - 2.6) / (12.0 - 2.6), 0.0, 1.0);

    // Lookup disk color from texture
    vec3 col = texture(colorMap, vec2(u, v)).rgb;
    return col;
}

// 🆕 Background uses cubemap skybox
vec3 getSkyboxColor(vec3 dir) {
    return texture(skybox, dir).rgb;
}

vec3 traceRay(vec3 pos, vec3 dir) {
    vec3 color = vec3(0.0);
    float alpha = 1.0;
    
    float STEP_SIZE = 0.1;
    dir *= STEP_SIZE;

    vec3 h = cross(pos, dir);
    float h2 = dot(h, h);

    float diskInnerR = R_S * 2.6;
    float diskOuterR = R_S * 12.0;
    vec3 oldPos = pos;

    for (int i = 0; i < 300; i++) {
        if (u_gravitational_lensing) {
            vec3 acc = accel(h2, pos);
            dir += acc;
        }

        if (dot(pos, pos) < R_S * R_S) {
            return vec3(0.0); // Event horizon
        }

        if (u_render_disk && crossesDisk(oldPos, pos, diskInnerR, diskOuterR)) {
            vec3 diskCol = getDiskColor(pos);
            color += diskCol * alpha * 0.4;
            alpha *= 0.7;
        }

        oldPos = pos;
        pos += dir;
    }

    // 🆕 Add realistic nebula background from skybox
    color += getSkyboxColor(normalize(dir)) * alpha;
    return color;
}

void main() {
    vec3 normalizedCamPos = u_camera_pos / 1.27e10;

    vec2 uv = (gl_FragCoord.xy / u_resolution.xy) * 2.0 - 1.0;
    uv.x *= u_resolution.x / u_resolution.y;

    float tanHalfFov = tan(radians(u_fov * 0.5));
    vec3 dir = normalize(vec3(uv.x * tanHalfFov, -uv.y * tanHalfFov, 1.0));
    dir = u_view_matrix * dir;

    vec3 color = traceRay(normalizedCamPos, dir);
    FragColor = vec4(color, 1.0);
}
