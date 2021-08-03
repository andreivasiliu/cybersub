#version 100

precision highp float;                                   

varying lowp vec2 uv;

uniform sampler2D wall_texture;
uniform sampler2D walls;
uniform vec2 walls_size;

void main() {
	if (texture2D(walls, uv) != vec4(0.0, 0.0, 0.0, 0.0)) {
		gl_FragColor = texture2D(wall_texture, fract(uv * walls_size)) * vec4(0.5, 0.5, 0.5, 1.0);
	} else {
		gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
	}
}
