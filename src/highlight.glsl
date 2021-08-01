#version 100

precision highp float;

varying lowp vec2 uv;

uniform sampler2D input_texture;
uniform vec2 input_resolution;
uniform float frame_y;
uniform float frame_height;

float lookup(vec2 p, float dx, float dy)
{
    float d = 1.0;
    vec2 uv = p.xy + vec2(dx * d, dy * d) / input_resolution;
    vec4 c = texture2D(input_texture, uv.xy);
	
    return c.a;
}

void main()
{
    float p_y = (uv.y / input_resolution.y * frame_height + frame_y / input_resolution.y);
    vec2 p = vec2(uv.x, p_y);

    float current_alpha = lookup(p, 0.0, 0.0);

    // Only affect transparent pixels; don't draw over the image.
    if (current_alpha != 0.0) {
        gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }

    float alpha = 0.0;

    alpha += lookup(p, -1.0, -1.0);
    alpha += lookup(p, -1.0,  0.0);
    alpha += lookup(p, -1.0,  1.0);
    alpha += lookup(p,  0.0, -1.0);
    alpha += lookup(p,  0.0,  1.0);
    alpha += lookup(p,  1.0, -1.0);
    alpha += lookup(p,  1.0,  0.0);
    alpha += lookup(p,  1.0,  1.0);

    float different = ceil(fract(alpha / 8.0));

	gl_FragColor = vec4(0.8, 0.4, 0.8, different);
}
