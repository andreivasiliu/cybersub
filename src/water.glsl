// Shamelessly stolen from https://www.shadertoy.com/view/MdlXz8
// Here just temporarily to test that shaders work; the sea water shader I have
// in my head looks way different (particles in 3 planes moving in parallax
// with the sub), but I don't know enough GLSL to do it yet.

#version 100

#define SHOW_TILING

#define TAU 6.28318530718
#define MAX_ITER 30

precision highp float;

uniform float iTime;
uniform vec2 iResolution;

varying lowp vec2 uv;

// out vec4 fragColor;
// in vec2 fragCoord;

vec4 mainImage( vec2 fragCoord ) 
{
	float time = iTime * .5+23.0;
    // uv should be the 0-1 uv of texture...
	vec2 uv = fragCoord.xy / iResolution.xy;
    
#ifdef SHOW_TILING
	vec2 p = mod(uv*TAU*2.0, TAU)-250.0;
#else
    vec2 p = mod(uv*TAU, TAU)-250.0;
#endif
	vec2 i = vec2(p);
	float c = 1.0;
	float inten = .005;

	for (int n = 0; n < MAX_ITER; n++) 
	{
		float t = time * (1.0 - (3.5 / float(n+1)));
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
	vec2 pixel = 2.0 / iResolution.xy;
	uv *= 2.0;

	float f = floor(mod(iTime*.5, 2.0)); 	// Flash value.
	vec2 first = step(pixel, uv) * f;		   	// Rule out first screen pixels and flash.
	uv  = step(fract(uv), pixel);				// Add one line of pixels per tile.
	colour = mix(colour, vec3(1.0, 0.0, 0.0), (uv.x + uv.y) * first.x * first.y); // Yellow line
	
	#endif
	vec4 fragColor = vec4(colour, 1.0);
	return fragColor;
}


void main()
{
	// mainImage(fragColor, fragCoord);
	gl_FragColor = mainImage(uv);
}
