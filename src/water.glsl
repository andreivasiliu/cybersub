#version 100

precision highp float;

varying vec2 uv;

uniform float enable_dust;
uniform float enable_caustics;
uniform sampler2D sea_dust;
uniform vec2 time_offset;
uniform vec2 camera_offset;
uniform float time;
uniform vec2 world_size;
uniform vec2 sea_dust_size;


#define TAU 6.28318530718
#define MAX_ITER 30

// Source: https://www.shadertoy.com/view/MdlXz8 by Dave Hoskins.
// Probably way too complex for what I need, so this is here just until I
// figure out enough GLSL to make my own.
vec4 caustics(vec2 dust_uv) {
	float scaled_time = time * 0.1 + 23.0;
    // uv should be the 0-1 uv of texture...
	highp vec2 uv = dust_uv;
    
    highp vec2 p = mod(uv*TAU, TAU)-250.0;
	highp vec2 i = vec2(p);
	float c = 1.0;
	float inten = .005;

	for (int n = 0; n < MAX_ITER; n++) 
	{
		float t = scaled_time * (1.0 - (3.5 / float(n+1)));
		i = p + vec2(cos(t - i.x) + sin(t + i.y), sin(t - i.y) + cos(t + i.x));
		c += 1.0/length(vec2(p.x / (sin(i.x+t)/inten),p.y / (cos(i.y+t)/inten)));
	}
	c /= float(MAX_ITER);
	c = 1.17-pow(c, 1.4);
	vec3 colour = vec3(pow(abs(c), 8.0));
    colour = clamp((colour + vec3(0.0078, 0.3569, 0.7529)) / 4.0, 0.0, 1.0);
    
	return vec4(colour, 1.0);
}

void main() {
	vec4 dust_color = vec4(0.0, 0.0, 0.0, 1.0);
	vec4 caustics_color = vec4(0.0, 0.0, 0.0, 0.0);
	highp vec2 uv = uv;
	vec2 dust_uv = fract(uv * sea_dust_size / world_size);

	if (enable_caustics == 1.0) {
		caustics_color = caustics(fract(dust_uv + time_offset / 3.0)) * 0.3;
	}

	if (enable_dust == 1.0) {
		vec4 a = texture2D(sea_dust, fract(dust_uv + time_offset / 1.0 + camera_offset * 0.2));
		vec4 b = texture2D(sea_dust, fract(dust_uv + time_offset / 1.5 + camera_offset * 1.5).yx);
		vec4 c = texture2D(sea_dust, fract(-(dust_uv + time_offset / 3.0 + camera_offset * 2.0)));

		dust_color = max(max(a, b), c);
	}

	vec4 background_color = vec4(0.0235, 0.0235, 0.1255, 0.0);
	gl_FragColor = background_color + dust_color + caustics_color;
}
