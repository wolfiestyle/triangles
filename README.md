# Triangles

This code approximates an image using only triangles. All work is done on the GPU using OpenGL drawing and compute shaders.

It works by drawing a fixed amount of random triangles (100 by default) and comparing them to the reference image via Mean Square Error. On each iteration we randomly mutate a single value from the triangle array and re-evaluate the error again. If it decreases we keep the change, otherwise we revert it. By brute forcing this at GPU speeds, we can get visible results pretty quickly.
