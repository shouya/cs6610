#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 n;

out vec2 uv_t; // in texture space
out vec3 pos_v; // in view space
out vec3 n_v; // in view space

uniform mat4 mv;
uniform mat3 mv_n; // for transforming vertex normals
uniform mat4 mvp;

void main()
{
    gl_Position = mvp * vec4(pos, 1.0);

    pos_v = (mv * vec4(pos, 1.0)).xyz;
    n_v = mv_n * n;

    uv_t = uv;
}
