#version 430
layout (local_size_x = 8, local_size_y = 8) in;

uniform sampler2D src;
uniform writeonly restrict image2D dest;

#define tid gl_LocalInvocationIndex
const uint block_size = gl_WorkGroupSize.x * gl_WorkGroupSize.y;

shared vec4 sdata[block_size];

vec4 fold_op(vec4, vec4);

void main() {
    ivec2 i = ivec2(2 * gl_WorkGroupSize.xy * gl_WorkGroupID.xy + gl_LocalInvocationID.xy);
    vec4 t0 = texelFetch(src, i, 0);
    vec4 t1 = texelFetch(src, i + ivec2(gl_WorkGroupSize.x, 0), 0);
    vec4 t2 = texelFetch(src, i + ivec2(0, gl_WorkGroupSize.y), 0);
    vec4 t3 = texelFetch(src, i + ivec2(gl_WorkGroupSize.xy), 0);
    sdata[tid] = fold_op(fold_op(fold_op(t0, t1), t2), t3);
    barrier();

    for (uint s = block_size/2; s > 0; s >>= 1) {
        if (tid < s) {
            sdata[tid] = fold_op(sdata[tid], sdata[tid + s]);
        }
        barrier();
    }

    if (tid == 0) {
        imageStore(dest, ivec2(gl_WorkGroupID.xy), sdata[0]);
    }
}
