#version 100

precision highp float;                                   

varying lowp vec2 uv;

uniform sampler2D new_sonar_texture;
uniform sampler2D old_sonar_texture;
uniform vec2 sonar_texture_size;
uniform float pulse;

void main() {
    vec2 position = 2.0 * (uv - vec2(0.5, 0.5));
    float distance_squared = position.x * position.x + position.y * position.y;
    float pulse_squared = pulse * pulse;
    float pulse_strength = fract(sqrt(distance_squared) - sqrt(pulse_squared));
    float signal_strength = 1.0 - (1.0 - pulse_strength) * (1.0 - pulse_strength);
    pulse_strength = pow(pulse_strength, 4.0);

    vec4 pulse_color = vec4(0.0, 0.2, 0.1, pulse_strength);

	vec4 texel_color;
    if (distance_squared < pulse_squared) {
        texel_color = texture2D(new_sonar_texture, uv);
    } else {
        texel_color = texture2D(old_sonar_texture, uv);
    }

    vec4 pixel_color;
    if (texel_color != vec4(0.0, 0.0, 0.0, 0.0)) {
		pixel_color = signal_strength * texel_color;
	} else {
        pixel_color = vec4(0.0, 0.0, 0.0, 0.0);
    }

    vec4 circle_color;
    if (signal_strength > 0.999) {
        circle_color = vec4(0.0, 0.4, 0.2, 0.5);
    } else {
        circle_color = vec4(0.0, 0.0, 0.0, 0.0);
    }

    if (distance_squared < 0.95) {
        gl_FragColor = clamp(pulse_color + pixel_color + circle_color, 0.0, 1.0);
    } else {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
    }
}
