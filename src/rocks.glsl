#version 100

precision highp float;                                   

varying lowp vec2 uv;

uniform sampler2D rocks_texture;
uniform sampler2D sea_rocks;
uniform vec2 sea_rocks_size;

void main() {
	vec4 texel_color = texture2D(sea_rocks, uv);

	if (texel_color != vec4(0.0, 0.0, 0.0, 0.0)) {
        float offset = ceil(texel_color.r * 15.9) - 1.0;
        vec2 rocks_uv = fract(uv * sea_rocks_size);
        vec2 frame_uv = vec2(rocks_uv.x, offset / 5.0 + rocks_uv.y / 5.0);

		gl_FragColor = texture2D(rocks_texture, frame_uv) * vec4(0.5, 0.5, 0.5, 1.0);
	} else {
		gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
	}
}
