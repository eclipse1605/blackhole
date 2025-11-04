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

// Texture uniforms for Ross Ning–style visuals
// textures
uniform sampler2D colorMap;
uniform samplerCube skybox;
// Dynamic quality controls (set from the host app)
uniform int u_max_iter;        // maximum march iterations
uniform float u_step_scale;    // multiplier applied to STEP_SIZE based on quality
uniform int u_noise_lod;       // noise LOD (effective max)

// --- Accretion disk parameters (tunable constants copied/approximated from RossNing)
const float ADISK_INNER = 2.6;
const float ADISK_OUTER = 12.0;
// make the disk visibly thicker in world units
const float ADISK_HEIGHT = 1.0;
const float ADISK_LIT = 1.0;
// vertical density exponent: lower -> slower falloff -> thicker appearance
const float ADISK_DENSITY_V = 2.0;
const float ADISK_DENSITY_H = 1.0;
const float ADISK_NOISE_SCALE = 1.0;
// Reduced noise LOD to cut down on expensive noise calls per-sample
const int   ADISK_NOISE_LOD = 2;
const float ADISK_SPEED = 0.5;
const float ADISK_PARTICLE = 1.0; // when <0.5, use particle-lite fallback

float hash(vec3 p) {
    p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

vec3 accel(float h2, vec3 pos) {
    float r2 = dot(pos, pos);
    // replace pow(r2, 2.5) with faster multiplies: r^5 = r2^2 * sqrt(r2)
    float r5 = r2 * r2 * sqrt(r2 + 1e-9);
    return -1.5 * h2 * pos / r5;
}


// --- Simplex noise (ported compactly from RossNing shader)
vec4 permute(vec4 x) { return mod(((x * 34.0) + 1.0) * x, 289.0); }
vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }
float snoise(vec3 v) {
    const vec2 C = vec2(1.0/6.0, 1.0/3.0);
    const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);

    // First corner
    vec3 i  = floor(v + dot(v, C.yyy));
    vec3 x0 = v - i + dot(i, C.xxx);

    // Other corners
    vec3 g = step(x0.yzx, x0.xyz);
    vec3 l = 1.0 - g;
    vec3 i1 = min(g.xyz, l.zxy);
    vec3 i2 = max(g.xyz, l.zxy);

    vec3 x1 = x0 - i1 + C.xxx;
    vec3 x2 = x0 - i2 + C.yyy;
    vec3 x3 = x0 - D.yyy;

    i = mod(i, 289.0);
    vec4 p = permute(permute(permute(i.z + vec4(0.0, i1.z, i2.z, 1.0))
               + i.y + vec4(0.0, i1.y, i2.y, 1.0))
               + i.x + vec4(0.0, i1.x, i2.x, 1.0));

    vec3 ns = 1.0/7.0 * D.wyz - D.xzx;
    vec4 j = p - 49.0 * floor(p * ns.z * ns.z);
    vec4 x_ = floor(j * ns.z);
    vec4 y_ = floor(j - 7.0 * x_);

    vec4 x = x_ * ns.x + ns.yyyy;
    vec4 y = y_ * ns.x + ns.yyyy;
    vec4 h = 1.0 - abs(x) - abs(y);

    vec4 b0 = vec4(x.xy, y.xy);
    vec4 b1 = vec4(x.zw, y.zw);

    vec4 s0 = floor(b0) * 2.0 + 1.0;
    vec4 s1 = floor(b1) * 2.0 + 1.0;
    vec4 sh = -step(h, vec4(0.0));

    vec4 a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    vec4 a1 = b1.xzyw + s1.xzyw * sh.zzww;

    vec3 p0 = vec3(a0.xy, h.x);
    vec3 p1 = vec3(a0.zw, h.y);
    vec3 p2 = vec3(a1.xy, h.z);
    vec3 p3 = vec3(a1.zw, h.w);

    vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2,p2), dot(p3,p3)));
    p0 *= norm.x; p1 *= norm.y; p2 *= norm.z; p3 *= norm.w;

    vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
    m = m * m;
    return 42.0 * dot(m*m, vec4(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
}

// volumetric accretion disk color accumulation per-march-step
// Uses a simple emission + Beer–Lambert attenuation so the disk absorbs
// light and progressively reduces the ray's alpha (transmittance).
void adiskColor(in vec3 pos, inout vec3 color, inout float alpha, in float step) {
    float innerRadius = ADISK_INNER;
    float outerRadius = ADISK_OUTER;

    // radial / vertical falloff
    float density = max(0.0, 1.0 - length(pos.xyz / vec3(outerRadius, ADISK_HEIGHT, outerRadius)));
    if (density < 0.001) return;

    density *= pow(1.0 - abs(pos.y) / ADISK_HEIGHT, ADISK_DENSITY_V);

    // mask out inside the innermost stable circular orbit
    density *= smoothstep(innerRadius, innerRadius * 1.1, length(pos.xz));
    if (density < 0.001) return;

    // spherical-ish coords for noise and texture lookup
    float rho = length(pos);
    float theta = atan(pos.z, pos.x);
    float phi = abs(pos.y);

    // radial UV for color map lookup (map colormap by radius)
    float v_radial = clamp((rho - innerRadius) / (outerRadius - innerRadius), 0.0, 1.0);

    // radial LOD and noise accumulation (dynamic LOD via uniform u_noise_lod)
    float noise = 1.0;
    for (int i = 0; i < 8; i++) {
        if (i >= u_noise_lod) break;
        float f = float(i*i);
        // add time-driven rotation to theta so the disk appears to spin
        float theta_t = theta + u_time * ADISK_SPEED * 0.5;
        noise *= 0.5 * snoise(vec3(rho, theta_t, phi) * f * ADISK_NOISE_SCALE) + 0.5;
        // small per-LOD offset to break repetition
        if (i % 2 == 0) theta += ADISK_SPEED * 0.01; else theta -= ADISK_SPEED * 0.01;
    }

    density *= 1.0 / pow(rho, ADISK_DENSITY_H);
    // tuneable visual scale factor (make larger so the disk is more visible)
    density *= 160.0;

    // Simple emission color from the disk (samples the provided colorMap)
    if (ADISK_PARTICLE < 0.5) {
    // particle fallback uses radial mapping for the color strip
    vec3 dustColor = texture(colorMap, vec2(v_radial, 0.5)).rgb;
        // emission
        vec3 emission = dustColor * density * 0.04 * abs(noise);
        // attenuation coefficient (controls how quickly light is absorbed)
        float sigma = 0.02;
        float tau = density * sigma * step;
        float trans = exp(-tau);
        color += (1.0 - trans) * emission * alpha;
        alpha *= trans;
        return;
    }

    // Sample colorMap radially (u = radius) so the colormap strip maps outward
    // from the inner radius to the outer radius.
    vec3 dustColor = texture(colorMap, vec2(v_radial, 0.5)).rgb;

    // emission scaled by density, lighting and noise
    vec3 emission = density * ADISK_LIT * dustColor * abs(noise);
    // attenuation coefficient (controls how quickly light is absorbed)
    // reduced so the disk contributes more visible emission per step
    float sigma = 0.02;
    float tau = density * sigma * step;
    float trans = exp(-tau);
    color += (1.0 - trans) * emission * alpha;
    alpha *= trans;
}

// Disk color now uses texture lookup
vec3 getDiskColor(vec3 pos) {
    float r = length(pos.xz);
    // Map radius to [0,1] range (inner radius = ADISK_INNER, outer = ADISK_OUTER)
    float v = clamp((r - ADISK_INNER) / (ADISK_OUTER - ADISK_INNER), 0.0, 1.0);
    // Lookup disk color radially from the colormap strip
    vec3 col = texture(colorMap, vec2(v, 0.5)).rgb;
    return col;
}

// Background uses cubemap skybox
vec3 getSkyboxColor(vec3 dir) {
    return texture(skybox, dir).rgb;
}

vec3 traceRay(vec3 pos, vec3 dir, vec3 viewDir) {
    vec3 color = vec3(0.0);
    float alpha = 1.0;

    // integrate ray direction as a unit vector; step is used to advance position
    // Base step size. We'll scale this adaptively by distance so far-away rays
    // take larger steps (fewer iterations) while preserving accuracy near the hole.
    const float STEP_SIZE = 0.06; // base step length (units)
    float dist = length(pos);
    // scale step with distance: near the hole use smaller step (0.6x), far away increase
    float distScale = clamp(dist / 4.0, 0.6, 3.0);
    float step = STEP_SIZE * distScale * u_step_scale;

    // rayDir is the current direction (per-unit) of the ray; do not scale it prematurely
    vec3 rayDir = normalize(dir);
    vec3 distortedViewDir = normalize(viewDir);

    // determine maximum travel distance for this ray based on starting distance
    float maxDist = length(pos) + ADISK_OUTER * 2.0;
    float traveled = 0.0;

    // Use a compile-time cap but break based on the uniform u_max_iter so the host
    // can reduce workload at runtime (quality presets).
    const int MAX_CAP = 2000;
    const int NORM_INTERVAL = 4; // renormalize direction every N iterations
    for (int i = 0; i < MAX_CAP; i++) {
        if (i >= u_max_iter) break;
        // recompute impact parameter each iteration using the unit ray direction
        vec3 h = cross(pos, rayDir);
        float h2 = dot(h, h);

        // accumulate disk color only for samples outside the horizon; pass
        // the current step so adiskColor can attenuate the ray (reduce alpha)
        if (u_render_disk && alpha > 0.001 && dot(pos, pos) >= R_S * R_S) {
            adiskColor(pos, color, alpha, step);
            // early out if the ray is almost fully attenuated
            if (alpha < 0.001) {
                return color;
            }
        }

        if (u_gravitational_lensing) {
            // compute acceleration (per-unit) and integrate it over this step
            vec3 acc = accel(h2, pos);
            // integrate acceleration to change direction: dv = a * dt (here dt ~= step)
            rayDir += acc * step;
            distortedViewDir += acc * step;
            // renormalize directions only every few iterations to save cost
            if (i % NORM_INTERVAL == 0) {
                rayDir = normalize(rayDir);
                distortedViewDir = normalize(distortedViewDir);
            }
        }

    // advance the ray by rayDir * step
        pos += rayDir * step;
        traveled += step;

        // If the ray moved into the event horizon during this step, stop
        // marching and return the accumulated color (light from before the
        // horizon can still reach the observer).
        if (dot(pos, pos) < R_S * R_S) {
            return color;
        }

        if (traveled > maxDist) break;
    }

    // Add realistic nebula background from skybox (use distorted view direction)
    color += getSkyboxColor(normalize(distortedViewDir)) * alpha;
    return color;
}

void main() {
    // camera position: use application-provided camera coordinates directly (camera uses shader units)
    vec3 normalizedCamPos = u_camera_pos;

    vec2 uv = (gl_FragCoord.xy / u_resolution.xy) * 2.0 - 1.0;
    uv.x *= u_resolution.x / u_resolution.y;

    float tanHalfFov = tan(radians(u_fov * 0.5));
    vec3 dir = normalize(vec3(uv.x * tanHalfFov, -uv.y * tanHalfFov, 1.0));
    dir = u_view_matrix * dir;
    vec3 viewDir = dir; // preserve original view direction for skybox sampling

    vec3 color = traceRay(normalizedCamPos, dir, viewDir);

    // Simple tone mapping + gamma to avoid extreme overexposure
    // Reinhard tone mapping
    color = color / (color + vec3(1.0));
    // gamma correction
    color = pow(color, vec3(1.0 / 2.2));

    FragColor = vec4(color, 1.0);
}
