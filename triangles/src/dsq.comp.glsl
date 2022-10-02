#version 430
layout (local_size_x = 16, local_size_y = 16) in;

uniform sampler2D src1;
uniform sampler2D src2;
uniform writeonly restrict image2D dest;

void main()
{
    ivec2 i = ivec2(gl_GlobalInvocationID.xy);
    vec4 diff = texelFetch(src1, i, 0) - texelFetch(src2, i, 0);
    imageStore(dest, i, diff * diff);
}
