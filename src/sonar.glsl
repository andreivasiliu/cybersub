#version 100

precision highp float;                                   

varying lowp vec2 uv;

uniform sampler2D new_sonar_texture;
uniform sampler2D old_sonar_texture;
uniform vec2 sonar_texture_size;
uniform float pulse;

vec4 pixel_by_strength(vec4 texel_color, float pulse_distance, bool old_signal) {
    if (texel_color == vec4(0.0, 0.0, 0.0, 0.0)) {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }

    if ((texel_color.g > pulse_distance) == old_signal) {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }

    vec4 signal_color = vec4(0.00, 0.64, 1.00, 1.0);
    vec4 recent_color = vec4(0.4, 0.4, 0.4, 1.0);

    float pixel_strength = texel_color.r;
    float pixel_signal = fract(texel_color.g - pulse_distance);
    float recent_boost = clamp((pixel_signal - 0.9) * 10.0, 0.0, 1.0) * 0.5;

    return clamp(pixel_signal * signal_color * pixel_strength + recent_boost * recent_color, 0.0, 1.0);
}

void main() {
    // Strangely, signals appear slightly sooner compared to the scanning circle
    // This is here until I figure out why
    float weird_number = 1.1;

    vec2 position = 2.0 * (uv - vec2(0.5, 0.5));
    float pixel_distance = length(position);
    float pulse_distance = pulse;
    float pulse_strength = fract(pixel_distance / weird_number - pulse_distance);

    if (pixel_distance > 0.95) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }

    vec4 pulse_color = vec4(0.0, 0.2, 0.1, pow(pulse_strength, 4.0));

    // Keep displaying old signals from the old scan rather than updating them in real-time
    vec4 old_color = pixel_by_strength(texture2D(new_sonar_texture, uv), pulse_distance, true);
    vec4 new_color = pixel_by_strength(texture2D(old_sonar_texture, uv), pulse_distance, false);

    vec4 pixel_color = max(old_color, new_color);

    vec4 circle_color;
    if (abs(pixel_distance - pulse_distance * weird_number) < 0.01) {
        circle_color = vec4(0.0, 0.4, 0.2, 0.5);
    } else {
        circle_color = vec4(0.0, 0.0, 0.0, 0.0);
    }

    gl_FragColor = clamp(pulse_color + pixel_color + circle_color, 0.0, 1.0);
}
