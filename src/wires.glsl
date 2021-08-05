#version 100

precision highp float;

varying lowp vec2 uv;

uniform vec2 grid_size;
uniform sampler2D sub_wires;
uniform sampler2D sub_signals;

void main() {
    vec3 wire_color = texture2D(sub_wires, uv).rgb;
    vec4 wire_signals = texture2D(sub_signals, uv);

    vec3 color;
    float signal;

    if (wire_color == vec3(1.0, 1.0, 0.0)) {
        // Orange
        color = vec3(0.80, 0.26, 0.12);
        signal = wire_signals.r;
    } else if (wire_color == vec3(0.0, 1.0, 1.0)) {
        // Brown
        color = vec3(0.22, 0.07, 0.03);
        signal = wire_signals.g;
    } else if (wire_color == vec3(0.0, 0.0, 1.0)) {
        // Blue
        color = vec3(0.1, 0.1, 0.4);
        signal = wire_signals.b;
    } else if (wire_color == vec3(0.0, 1.0, 0.0)) {
        // Green
        color = vec3(0.1, 0.4, 0.1);
        signal = wire_signals.a;
    } else {
        // Nothing
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }

    gl_FragColor = vec4(color + vec3(0.2, 0.2, 0.2) * signal, 1.0);
}
