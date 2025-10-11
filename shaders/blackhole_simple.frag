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

float hash(vec3 p) {
    p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

float noise(vec3 x) {
    vec3 i = floor(x);
    vec3 f = fract(x);
    f = f * f * (3.0 - 2.0 * f);
    
    return mix(mix(mix(hash(i + vec3(0,0,0)), hash(i + vec3(1,0,0)), f.x),
                   mix(hash(i + vec3(0,1,0)), hash(i + vec3(1,1,0)), f.x), f.y),
               mix(mix(hash(i + vec3(0,0,1)), hash(i + vec3(1,0,1)), f.x),
                   mix(hash(i + vec3(0,1,1)), hash(i + vec3(1,1,1)), f.x), f.y), f.z);
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

vec3 getDiskColor(vec3 pos, float time) {
    float r = length(pos.xz);
    float angle = atan(pos.z, pos.x);
    
    float temp = 1.0 / pow(r / R_S, 0.5);
    
    vec3 noiseCoord = vec3(angle * 2.0, r * 2.0, time * 0.2);
    float n = noise(noiseCoord);
    
    vec3 coldColor = vec3(1.0, 0.4, 0.1);
    vec3 hotColor = vec3(1.0, 0.95, 0.8);
    vec3 baseColor = mix(coldColor, hotColor, temp);
    
    return baseColor * (0.5 + n * 1.5) * temp;
}

vec3 getSkyboxColor(vec3 dir) {
    float stars = pow(noise(dir * 20.0), 15.0);
    vec3 nebula = vec3(0.05, 0.02, 0.1) * noise(dir * 3.0);
    return nebula + vec3(stars);
}

vec3 traceRay(vec3 pos, vec3 dir) {
    vec3 color = vec3(0.0);
    float alpha = 1.0;
    
    float STEP_SIZE = 0.1;
    dir *= STEP_SIZE;
    
    // Calculate angular momentum
    vec3 h = cross(pos, dir);
    float h2 = dot(h, h);
    
    float diskInnerR = R_S * 2.6;
    float diskOuterR = R_S * 12.0;
    vec3 oldPos = pos;
    
    int maxSteps = 300;
    for (int i = 0; i < maxSteps; i++) {
        if (u_gravitational_lensing) {
            // Apply gravitational acceleration
            vec3 acc = accel(h2, pos);
            dir += acc;
        }
        
        // Check event horizon
        if (dot(pos, pos) < R_S * R_S) {
            return color;  // Return black if hit event horizon
        }
        
        // Accumulate disk color (volume rendering style)
        if (u_render_disk && crossesDisk(oldPos, pos, diskInnerR, diskOuterR)) {
            vec3 diskCol = getDiskColor(pos, u_time);
            color += diskCol * alpha * 0.3;  // Semi-transparent accumulation
            alpha *= 0.7;  // Reduce alpha for next layer
        }
        
        oldPos = pos;
        pos += dir;
    }
    
    color += getSkyboxColor(normalize(dir)) * alpha;
    return color;
}

void main() {
    // Calculate normalized camera position (in units of R_S)
    // Original camera is at ~6e10 m, R_S is ~1.27e10 m, so normalized is ~4.7
    vec3 normalizedCamPos = u_camera_pos / 1.27e10;  // Normalize to R_S units
    
    // Calculate ray direction
    vec2 uv = (gl_FragCoord.xy / u_resolution.xy) * 2.0 - 1.0;
    uv.x *= u_resolution.x / u_resolution.y;
    
    float tanHalfFov = tan(radians(u_fov * 0.5));
    vec3 dir = normalize(vec3(uv.x * tanHalfFov, -uv.y * tanHalfFov, 1.0));
    dir = u_view_matrix * dir;
    
    // Trace ray
    vec3 color = traceRay(normalizedCamPos, dir);
    
    FragColor = vec4(color, 1.0);
}
