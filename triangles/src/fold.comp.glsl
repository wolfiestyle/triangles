#version 430
layout (local_size_x = 16, local_size_y = 16) in;

uniform sampler2D src;
uniform writeonly restrict image2D dest;

const uint tid = gl_LocalInvocationIndex;
const uint block_size = gl_WorkGroupSize.x * gl_WorkGroupSize.y;

shared vec4 sdata[block_size];

vec4 fold_op(vec4, vec4);

void main()
{
    ivec2 i = ivec2(2 * gl_WorkGroupSize.xy * gl_WorkGroupID.xy + gl_LocalInvocationID.xy);
    vec4 t0 = texelFetch(src, i, 0);
    vec4 t1 = texelFetch(src, i + ivec2(gl_WorkGroupSize.x, 0), 0);
    vec4 t2 = texelFetch(src, i + ivec2(0, gl_WorkGroupSize.y), 0);
    vec4 t3 = texelFetch(src, i + ivec2(gl_WorkGroupSize.xy), 0);
    sdata[tid] = fold_op(fold_op(fold_op(t0, t1), t2), t3);
    barrier();

    if (block_size >= 512) { if (tid < 256) { sdata[tid] = fold_op(sdata[tid], sdata[tid + 256]); } barrier(); }
    if (block_size >= 256) { if (tid < 128) { sdata[tid] = fold_op(sdata[tid], sdata[tid + 128]); } barrier(); }
    if (block_size >= 128) { if (tid < 64)  { sdata[tid] = fold_op(sdata[tid], sdata[tid +  64]); } barrier(); }

    if (tid < 32)
    {
        if (block_size >= 64) sdata[tid] = fold_op(sdata[tid], sdata[tid + 32]);
        if (block_size >= 32) sdata[tid] = fold_op(sdata[tid], sdata[tid + 16]);
        if (block_size >= 16) sdata[tid] = fold_op(sdata[tid], sdata[tid + 8]);
        if (block_size >= 8)  sdata[tid] = fold_op(sdata[tid], sdata[tid + 4]);
        if (block_size >= 4)  sdata[tid] = fold_op(sdata[tid], sdata[tid + 2]);
        if (block_size >= 2)  sdata[tid] = fold_op(sdata[tid], sdata[tid + 1]);
    }

    if (tid == 0)
        imageStore(dest, ivec2(gl_WorkGroupID.xy), sdata[0]);
}
