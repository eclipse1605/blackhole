#version 330 core

in vec2 vTexCoord;
out vec4 FragColor;

// Constants
const float PI = 3.14159265359;
const float EPSILON = 0.0001;
const float INFINITY = 1e30;
const float C = 299792458.0;
const float G = 6.67430e-11;

// Black hole parameters (Sagittarius A*)
const float BH_MASS = 8.54e36;
const float R_S = 2.0 * G * BH_MASS / (C * C); // Schwarzschild radius: ~1.269e10 m
const float D_LAMBDA = 1e7; // Integration step size

// Uniforms
uniform vec2 u_resolution;
uniform float u_time;
uniform vec3 u_camera_pos;
uniform mat3 u_view_matrix;
uniform float u_fov;
uniform bool u_render_disk;
uniform bool u_gravitational_lensing;

// Simplex noise for accretion disk
vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec4 mod289(vec4 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec4 permute(vec4 x) { return mod289(((x * 34.0) + 1.0) * x); }
vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }

float snoise(vec3 v) {
    const vec2 C = vec2(1.0/6.0, 1.0/3.0);
    const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);
    
    vec3 i  = floor(v + dot(v, C.yyy));
    vec3 x0 = v - i + dot(i, C.xxx);
    
    vec3 g = step(x0.yzx, x0.xyz);
    vec3 l = 1.0 - g;
    vec3 i1 = min(g.xyz, l.zxy);
    vec3 i2 = max(g.xyz, l.zxy);
    
    vec3 x1 = x0 - i1 + C.xxx;
    vec3 x2 = x0 - i2 + C.yyy;
    vec3 x3 = x0 - D.yyy;
    
    i = mod289(i);
    vec4 p = permute(permute(permute(
                i.z + vec4(0.0, i1.z, i2.z, 1.0))
              + i.y + vec4(0.0, i1.y, i2.y, 1.0))
              + i.x + vec4(0.0, i1.x, i2.x, 1.0));
    
    float n_ = 0.142857142857;
    vec3 ns = n_ * D.wyz - D.xzx;
    
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
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;
    
    vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
    m = m * m;
    return 42.0 * dot(m*m, vec4(dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3)));
}

// Convert Cartesian to spherical coordinates
vec3 toSpherical(vec3 pos) {
    float r = length(pos);
    float theta = acos(clamp(pos.y / r, -1.0, 1.0));
    float phi = atan(pos.z, pos.x);
    return vec3(r, theta, phi);
}

// Convert spherical to Cartesian
vec3 toCartesian(vec3 sph) {
    float r = sph.x;
    float theta = sph.y;
    float phi = sph.z;
    return vec3(
        r * sin(theta) * cos(phi),
        r * cos(theta),
        r * sin(theta) * sin(phi)
    );
}

// Ray structure for geodesic integration
struct Ray {
    vec3 pos;      // Cartesian position
    vec3 dir;      // Cartesian direction (velocity)
    float r;       // Radial distance
    float theta;   // Polar angle
    float phi;     // Azimuthal angle
    float dr;      // d(r)/d(lambda)
    float dtheta;  // d(theta)/d(lambda)
    float dphi;    // d(phi)/d(lambda)
    float E;       // Energy constant
    float L;       // Angular momentum
};

// Initialize a ray from position and direction
Ray initRay(vec3 pos, vec3 dir) {
    Ray ray;
    ray.pos = pos;
    ray.dir = normalize(dir);
    
    // Convert to spherical coordinates
    ray.r = length(pos);
    ray.theta = acos(clamp(pos.y / ray.r, -1.0, 1.0));
    ray.phi = atan(pos.z, pos.x);
    
    // Calculate spherical velocity components
    float sinTheta = sin(ray.theta);
    float cosTheta = cos(ray.theta);
    float sinPhi = sin(ray.phi);
    float cosPhi = cos(ray.phi);
    
    ray.dr = sinTheta * cosPhi * dir.x + cosTheta * dir.y + sinTheta * sinPhi * dir.z;
    ray.dtheta = (cosTheta * cosPhi * dir.x - sinTheta * dir.y + cosTheta * sinPhi * dir.z) / ray.r;
    ray.dphi = (-sinPhi * dir.x + cosPhi * dir.z) / (ray.r * sinTheta + EPSILON);
    
    // Calculate conserved quantities
    ray.L = ray.r * ray.r * sin(ray.theta) * ray.dphi;
    float f = 1.0 - R_S / ray.r;
    float dt_dlambda = sqrt((ray.dr * ray.dr) / f + ray.r * ray.r * (ray.dtheta * ray.dtheta + sinTheta * sinTheta * ray.dphi * ray.dphi));
    ray.E = f * dt_dlambda;
    
    return ray;
}

// Geodesic equations RHS for Schwarzschild metric
void geodesicRHS(in Ray ray, out vec3 d1, out vec3 d2) {
    float r = ray.r;
    float theta = ray.theta;
    float dr = ray.dr;
    float dtheta = ray.dtheta;
    float dphi = ray.dphi;
    
    float f = 1.0 - R_S / r;
    float dt_dlambda = ray.E / f;
    float sinTheta = sin(theta);
    float cosTheta = cos(theta);
    
    // d/dlambda of (r, theta, phi)
    d1 = vec3(dr, dtheta, dphi);
    
    // d/dlambda of (dr, dtheta, dphi)
    d2.x = -(R_S / (2.0 * r * r)) * f * dt_dlambda * dt_dlambda
         + (R_S / (2.0 * r * r * f)) * dr * dr
         + r * (dtheta * dtheta + sinTheta * sinTheta * dphi * dphi);
         
    d2.y = -2.0 * dr * dtheta / r + sinTheta * cosTheta * dphi * dphi;
    
    d2.z = -2.0 * dr * dphi / r - 2.0 * cosTheta / (sinTheta + EPSILON) * dtheta * dphi;
}

// RK4 integration step
void rk4Step(inout Ray ray, float dLambda) {
    vec3 k1a, k1b, k2a, k2b, k3a, k3b, k4a, k4b;
    Ray tempRay;
    
    // k1
    geodesicRHS(ray, k1a, k1b);
    
    // k2
    tempRay = ray;
    tempRay.r += dLambda * 0.5 * k1a.x;
    tempRay.theta += dLambda * 0.5 * k1a.y;
    tempRay.phi += dLambda * 0.5 * k1a.z;
    tempRay.dr += dLambda * 0.5 * k1b.x;
    tempRay.dtheta += dLambda * 0.5 * k1b.y;
    tempRay.dphi += dLambda * 0.5 * k1b.z;
    geodesicRHS(tempRay, k2a, k2b);
    
    // k3
    tempRay = ray;
    tempRay.r += dLambda * 0.5 * k2a.x;
    tempRay.theta += dLambda * 0.5 * k2a.y;
    tempRay.phi += dLambda * 0.5 * k2a.z;
    tempRay.dr += dLambda * 0.5 * k2b.x;
    tempRay.dtheta += dLambda * 0.5 * k2b.y;
    tempRay.dphi += dLambda * 0.5 * k2b.z;
    geodesicRHS(tempRay, k3a, k3b);
    
    // k4
    tempRay = ray;
    tempRay.r += dLambda * k3a.x;
    tempRay.theta += dLambda * k3a.y;
    tempRay.phi += dLambda * k3a.z;
    tempRay.dr += dLambda * k3b.x;
    tempRay.dtheta += dLambda * k3b.y;
    tempRay.dphi += dLambda * k3b.z;
    geodesicRHS(tempRay, k4a, k4b);
    
    // Update ray
    ray.r += (dLambda / 6.0) * (k1a.x + 2.0 * k2a.x + 2.0 * k3a.x + k4a.x);
    ray.theta += (dLambda / 6.0) * (k1a.y + 2.0 * k2a.y + 2.0 * k3a.y + k4a.y);
    ray.phi += (dLambda / 6.0) * (k1a.z + 2.0 * k2a.z + 2.0 * k3a.z + k4a.z);
    ray.dr += (dLambda / 6.0) * (k1b.x + 2.0 * k2b.x + 2.0 * k3b.x + k4b.x);
    ray.dtheta += (dLambda / 6.0) * (k1b.y + 2.0 * k2b.y + 2.0 * k3b.y + k4b.y);
    ray.dphi += (dLambda / 6.0) * (k1b.z + 2.0 * k2b.z + 2.0 * k3b.z + k4b.z);
    
    // Update Cartesian position
    ray.pos = toCartesian(vec3(ray.r, ray.theta, ray.phi));
}

// Check if ray hit the event horizon
bool hitEventHorizon(Ray ray) {
    return ray.r <= R_S * 1.01; // Small margin for numerical stability
}

// Check if ray crosses the accretion disk
bool crossesDisk(vec3 oldPos, vec3 newPos, float innerR, float outerR) {
    // Check if crossed equatorial plane (y = 0)
    bool crossed = (oldPos.y * newPos.y < 0.0);
    if (!crossed) return false;
    
    // Check if within disk radius
    float r = length(newPos.xz);
    return (r >= innerR && r <= outerR);
}

// Get accretion disk color with procedural noise
vec3 getDiskColor(vec3 pos, float time) {
    float r = length(pos.xz);
    vec3 spherical = toSpherical(pos);
    
    // Temperature gradient (hotter near center)
    float temp = 1.0 / pow(r / R_S, 0.75);
    
    // Procedural noise for turbulence
    vec3 noiseCoord = vec3(spherical.z * 2.0, spherical.x / R_S, time * 0.5);
    float noise = 0.0;
    float amplitude = 1.0;
    float frequency = 1.0;
    
    for (int i = 0; i < 5; i++) {
        noise += amplitude * (snoise(noiseCoord * frequency) * 0.5 + 0.5);
        amplitude *= 0.5;
        frequency *= 2.0;
        noiseCoord.x += time * 0.1 * float(i % 2 == 0 ? 1 : -1);
    }
    
    // Color based on temperature (blackbody-ish)
    vec3 coldColor = vec3(1.0, 0.3, 0.1);  // Reddish
    vec3 hotColor = vec3(1.0, 0.9, 0.6);   // Yellowish-white
    vec3 baseColor = mix(coldColor, hotColor, temp * 0.7);
    
    // Apply noise
    float intensity = temp * noise * 2.0;
    return baseColor * intensity;
}

// Simple skybox (procedural starfield)
vec3 getSkyboxColor(vec3 dir) {
    // Create a procedural starfield
    vec3 noiseInput = dir * 50.0;
    float stars = 0.0;
    
    // Multiple octaves for star density
    for (int i = 0; i < 3; i++) {
        float n = snoise(noiseInput * pow(2.0, float(i)));
        stars += pow(max(n, 0.0), 20.0 - float(i) * 5.0);
    }
    
    // Nebula-like background
    float nebula = snoise(dir * 2.0) * 0.5 + 0.5;
    vec3 nebulaColor = vec3(0.1, 0.05, 0.2) * nebula * 0.3;
    
    // Combine stars and nebula
    vec3 color = nebulaColor + vec3(stars) * vec3(1.0, 0.9, 0.8);
    
    return color;
}

// Main raytracing function
vec3 traceRay(vec3 origin, vec3 direction) {
    Ray ray = initRay(origin, direction);
    
    vec3 oldPos = ray.pos;
    vec3 color = vec3(0.0);
    bool hitSomething = false;
    
    // Accretion disk parameters
    float diskInnerR = R_S * 3.0;  // Inner radius (innermost stable circular orbit ~3 Rs)
    float diskOuterR = R_S * 10.0; // Outer radius
    
    // Raytrace loop
    int maxSteps = 300;
    for (int i = 0; i < maxSteps; i++) {
        // Check event horizon
        if (hitEventHorizon(ray)) {
            color = vec3(0.0); // Black
            hitSomething = true;
            break;
        }
        
        // Check disk intersection
        if (u_render_disk && crossesDisk(oldPos, ray.pos, diskInnerR, diskOuterR)) {
            color = getDiskColor(ray.pos, u_time);
            hitSomething = true;
            break;
        }
        
        // Escape condition
        if (ray.r > INFINITY * 0.1) {
            color = getSkyboxColor(normalize(ray.pos));
            break;
        }
        
        oldPos = ray.pos;
        
        // Integrate geodesic
        if (u_gravitational_lensing) {
            rk4Step(ray, D_LAMBDA);
        } else {
            // Straight line for comparison
            ray.pos += direction * D_LAMBDA;
            ray.r = length(ray.pos);
        }
    }
    
    return color;
}

void main() {
    // Calculate ray direction in screen space
    vec2 uv = (gl_FragCoord.xy / u_resolution.xy) * 2.0 - 1.0;
    uv.x *= u_resolution.x / u_resolution.y; // Correct aspect ratio
    
    float tanHalfFov = tan(radians(u_fov * 0.5));
    vec3 dir = normalize(vec3(uv.x * tanHalfFov, -uv.y * tanHalfFov, 1.0));
    
    // Transform direction by view matrix
    dir = u_view_matrix * dir;
    
    // Trace the ray
    vec3 color = traceRay(u_camera_pos, dir);
    
    FragColor = vec4(color, 1.0);
}
