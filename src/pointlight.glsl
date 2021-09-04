#version 100

precision lowp float;

varying lowp vec2 uv;

uniform vec2 pointlight_size;
uniform vec2 pointlight_position;

void main() {
    // vec2 uv = gl_FragCoord.xy;
    // vec2 uv = gl_FragCoord.xy / screen_size;
    // vec2 light = pointlight_position / screen_size;
    // light.y = 1.0 - light.y;
    // uv.y *= screen_size.y / screen_size.x;
    // light.y *= screen_size.y / screen_size.x;
    // float intensity = clamp(1.0 - length(light - uv) * 5.0, 0.0, 1.0);

    // vec2 pointlight_position = pointlight_position;
    // pointlight_position.y = 1.0 - pointlight_position.y;

    float pixel_distance = length((gl_FragCoord.xy - pointlight_position) / pointlight_size);

    gl_FragColor = vec4(1.0, 1.0, 1.0, clamp(1.0 - pixel_distance, 0.0, 1.0));
}
