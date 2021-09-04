#version 100

precision lowp float;

varying lowp vec2 uv;

uniform sampler2D shadows;
uniform sampler2D screen;

void main() {
    vec4 shadow_pixel = texture2D(shadows, uv);

    // Macroquad flips render targets upside-down
    vec4 sceen_pixel = texture2D(screen, vec2(uv.x, 1.0-uv.y));

    gl_FragColor = vec4(shadow_pixel.rgb * sceen_pixel.rgb, 1.0);
}
