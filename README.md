# Projects for CS 5610/6610

This repository contains my code for the assignments of the online course [Interactive Computer Graphics](https://graphics.cs.utah.edu/courses/cs6610/spring2021/) by Cem Yuksel.

## Building

I implemented the projects using Rust. The core dependencies are [winit](https://crates.io/crates/winit) (window creation and events), [glium](https://crates.io/crates/glium) (OpenGL binding), and [glam](https://crates.io/crates/glam) (math). The projects are organized into individual crates in a shared workspace.

To build and run each project, go to the project directory and run:

```sh
cd project-01-hello-world
cargo run --release
```

Further information about controls and extra features can be found in each project's README.

## Notes

My notes are available in the [notes.org](notes.org) file.

## Project demos

### Project 01

Animating background with varying clear color.

https://github.com/user-attachments/assets/b8cc11b5-b1e2-460e-b5cc-8c4d6dd717de

### Project 02

Rendering the vertex positions loaded from a Wavefront .obj file.

https://github.com/user-attachments/assets/94a2ec6c-101d-4051-b88a-680c994f06b8

### Project 03

Rotating teapot with Blinn shading.

https://github.com/user-attachments/assets/29e148a5-db1b-4c00-8df3-71d8553f055f

Visualizing the algorithmically generated triangle-strips from the mesh data.

https://github.com/user-attachments/assets/e4d82441-fea4-479a-9ba8-a914256ce88a

### Project 04

Rendering a large textured model with ~1M triangles processed into triangle strips.

https://github.com/user-attachments/assets/ecec7ea0-a2fd-4ca8-951e-313a0917e6f1

### Project 05

The yoda model rendered into a texture, which is in turn used as the texture of the teapot model.

https://github.com/user-attachments/assets/e8d0261c-2202-40a4-8675-b0efa0a9b00c

### Project 06

Two reflective teapots using cubemap reflection, a regular textured teapot, plus a flat reflective floor with computed ripples. Each reflective object also reflects each other.

https://github.com/user-attachments/assets/b50b431c-6cb5-49d2-ae8a-4044eea74f91

If you look closely you may notice the small delay in the reflection update in the reflective teapots. This because to I only update the cubemaps one face at a time. This is a common optimization technique I played with.

### Project 07

Shadow implemented using a shadow map.

https://github.com/user-attachments/assets/27757832-67ff-4e5e-b66f-388f85bf99d2

This demo shows how the shadow map changes corresponding to the changes in light position.

https://github.com/user-attachments/assets/37256a0b-1c4d-43e1-a346-3b277b49d58f

### Project 08

A single quad tessellated on the GPU with displacement map. This demo showcase the result of varying displacement levels, tessellation levels, and view angles.

https://github.com/user-attachments/assets/cc31ccf6-2946-4045-9d75-3577aab0e688

The same single quad render with displacement map. Instead of tessellation I implemented a per-fragment displacement using parallax mapping. The demo showcases the varying displacement levels and number of tracing steps in parallax mapping.

https://github.com/user-attachments/assets/ad37a59b-629c-4b66-8e7f-5f4bd0e7b49d
