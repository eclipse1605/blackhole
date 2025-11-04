# Blackhole Simulator

## Project Overview

- Interactive 3D simulator of photon paths and accretion-disk appearance around a Schwarzschild and Kerr black hole.
- Built in Rust for safety and performance; GLFW for windowing/input and OpenGL context.
- Real-time or near-real-time rendering using GPU fragment shaders for ray-tracing/ray-marching; CPU fallback for accurate geodesic integration.

## Features
- camera controls
- adjustable black hole mass & spin flag (Schwarzchild or Kerr)
- accretion disk texture
- relativistic Doppler & gravitational redshift
- photon sphere visualisation
- live FPS/performance stats

## Core CG concepts used
- Per-pixel ray generation in camera coordinates; trace photons by integrating geodesic (CPU) or approximate with shader ray marching.
- Shaders compute ray directions, sample accretion disk textures, and evaluate colour shifts from relativistic effects.
- Convert screen coordinates → camera space → initial 4-velocity for geodesic integration.
- Apply gravitational redshift and Doppler shift → convert to HDR float colour → tone mapping → gamma correction for display.
- Multisampling or TAA; LOD for disk textures; adaptive step size in integrator.
- Draw photon paths (polylines), render event horizon/photon sphere wireframes, show effective-potential graphs, and impact-parameter slider.
- GPU fragment-parallel processing for image generation; CPU multithreading for background-heavy integrator tasks or data prep.
 
## Running on a discrete GPU (Linux / hybrid systems)

If your machine has both an integrated GPU (iGPU) and a discrete GPU (dGPU), Linux may run the app on the iGPU by default which can be much slower. The app creates a regular GLFW/OpenGL context and will use whichever GL implementation the X/Wayland session exposes.

To force the process to use the discrete GPU you can start the program with one of the following environment variables depending on your driver stack:

- For Mesa-based drivers (AMD or Intel + discrete AMD):

```bash
DRI_PRIME=1 cargo run --release
```

- For NVIDIA proprietary drivers with PRIME render offload:

```bash
__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia cargo run --release
```

After the OpenGL context is created the program prints the GL vendor/renderer/version to stderr so you can confirm which GPU is active. Look for a line like:

```
OpenGL vendor: NVIDIA Corporation | renderer: NVIDIA GeForce ... | version: 4.x.x
```

If it shows something like `Intel` or `Mesa`/`AMD` you are on the integrated GPU; if it shows `NVIDIA` or a discrete device name then the dGPU is active.

If you want us to add automatic detection or a CLI flag to request a specific backend, tell me and I can implement it (low-risk change).
