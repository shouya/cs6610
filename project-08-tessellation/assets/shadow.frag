#version 330 core

// we don't do anything in shadow fragment shader. the only thing we
// need to build the shadow map is the depth value generated in the
// vertex shader, and interpolated over the rasterized mesh.

void main() {}
