#version 430

in vec2 position;
in vec4 color;

out vec4 vcolor;

void main()
{
    vcolor = color;
    gl_Position = vec4(position * 2.0 - 1.0, 0.0, 1.0);
}
