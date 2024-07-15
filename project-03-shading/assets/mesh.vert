#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 uv;
layout(location = 2) in vec3 n;

uniform mat4 mv;
uniform mat3 mv_n; // for transforming vertex normals
uniform mat4 mvp;

out vec3 out_uv; // in texture space
out vec3 out_pos; // in view space
out vec3 out_n; // in view space

void main()
{
    gl_Position = mvp * vec4(pos, 1.0);

    out_pos = (mv * vec4(pos, 1.0)).xyz;
    out_n = mv_n * n;

    out_uv = uv;
}
