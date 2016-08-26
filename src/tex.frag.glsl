#version 430

uniform sampler2D tex;

in vec2 vtexc;

out vec4 frag_color;

void main()
{
    frag_color = texture(tex, vtexc);
}
