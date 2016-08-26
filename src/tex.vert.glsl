#version 430

in vec2 coord;

out vec2 vtexc;

void main()
{
    vtexc = coord;
    vec2 vpos = vec2(coord.x, 1.0 - coord.y) * 2.0 - 1.0;
    gl_Position = vec4(vpos, 0.0, 1.0);
}
