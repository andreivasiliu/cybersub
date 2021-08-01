#version 100

precision highp float;

varying lowp vec2 uv;

uniform vec3 wire_color;
uniform float signal;
uniform sampler2D wires_texture;

void main()
{
    if (texture2D(wires_texture, uv).rgb == wire_color) {
        vec3 color;
        if (wire_color == vec3(1.0, 1.0, 0.0)) {
            // Orange
            color = vec3(0.80, 0.26, 0.12);
        } else if (wire_color == vec3(0.0, 1.0, 1.0)) {
            // Brown
            color = vec3(0.22, 0.07, 0.03);
        } else if (wire_color == vec3(0.0, 0.0, 1.0)) {
            // Blue
            color = vec3(0.1, 0.1, 0.4);
        } else if (wire_color == vec3(0.0, 1.0, 0.0)) {
            // Green
            color = vec3(0.1, 0.4, 0.1);
        }

        gl_FragColor = vec4(color + vec3(0.2, 0.2, 0.2) * signal, 1.0);
    } else {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
    }
}
