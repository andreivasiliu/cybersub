#version 100

precision highp float;

varying lowp vec2 uv;

uniform sampler2D sea_dust;
uniform vec2 time_offset;
uniform vec2 camera_offset;
uniform float time;
uniform vec2 resolution;


#define TAU 6.28318530718
#define MAX_ITER 30

vec4 mainImage( vec2 fragCoord ) 
{
	float scaled_time = time * 0.1 + 23.0;
    // uv should be the 0-1 uv of texture...
	vec2 uv = fragCoord.xy / resolution.xy;
    
    vec2 p = mod(uv*TAU, TAU)-250.0;
	vec2 i = vec2(p);
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
    // colour = clamp(colour + vec3(0.1608, 0.0118, 0.2784), 0.0, 1.0);
    colour = clamp((colour + vec3(0.0078, 0.3569, 0.7529)) / 4.0, 0.0, 1.0);
    

	#ifdef SHOW_TILING
	// Flash tile borders...
	vec2 pixel = 2.0 / resolution.xy;
	uv *= 2.0;

	float f = floor(mod(time*.5, 2.0)); 	// Flash value.
	vec2 first = step(pixel, uv) * f;		   	// Rule out first screen pixels and flash.
	uv  = step(fract(uv), pixel);				// Add one line of pixels per tile.
	colour = mix(colour, vec3(1.0, 0.0, 0.0), (uv.x + uv.y) * first.x * first.y); // Yellow line
	
	#endif
	vec4 fragColor = vec4(colour, 1.0);
	return fragColor;
}

void main() {
	vec4 a = texture2D(sea_dust, fract(uv * 6.0 + time_offset / 1.0 + camera_offset * 0.2));
	vec4 b = texture2D(sea_dust, fract(uv * 6.0 + vec2(0.5, 0.2) + time_offset / 1.5 + camera_offset * 1.5));
	vec4 c = texture2D(sea_dust, fract(uv * 6.0 + vec2(0.2, 0.5) + time_offset / 3.0 + camera_offset * 2.0));
	gl_FragColor = max(max(a, b), c) + vec4(0.0235, 0.0235, 0.1255, 0.0) + mainImage(fract(uv + time_offset / 3.0)) * 0.2;
}
