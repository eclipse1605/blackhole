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
