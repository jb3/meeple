/// Northern lights: curtains hung in the world as continuous ribbons, built in
/// the vertex shader from nothing but the curtains' placing (a few uniforms) and
/// the instance number: one instance is one slice of one curtain, a quad from
/// the foot of the curtain to its top. Neighbouring slices share their edges, so
/// together they make one smooth ribbon.
pub const AURORA_VS: &str = r#"#version 300 es
precision highp float;
uniform mat4 u_vp;
uniform float u_time;
uniform int u_n;
// per curtain: (centre x, centre z, direction x, direction z), then (length, phase, base height, 0)
uniform vec4 u_c[16];
out vec3 v_p;
const int SLICES = 120;
float rayAt(float s, float ph, float t) {
    return 0.5 + 0.5 * sin(s * 0.55 + sin(s * 0.07 - t * 0.3 + ph) * 3.0 - t * 0.5);
}
void main() {
    int id = gl_InstanceID;
    int ci = id / SLICES;
    int j = id - ci * SLICES;
    vec4 a = u_c[ci * 2];
    vec4 b = u_c[ci * 2 + 1];
    float len = b.x;
    float ph = b.y;
    vec2 dir = a.zw;
    vec2 side = vec2(-dir.y, dir.x);
    float t = u_time;
    float s = ((float(j + (gl_VertexID & 1)) / float(SLICES)) - 0.5) * len;
    float bend = 16.0 * sin(s * 0.032 + ph) + 7.0 * sin(s * 0.09 - t * 0.04 + ph);
    vec2 p = a.xy + dir * s + side * bend;
    float ends = smoothstep(0.0, 30.0, len * 0.5 - abs(s));
    float ray = rayAt(s, ph, t);
    float pulse = 0.55 + 0.45 * sin(s * 0.05 - t * 0.35 + ph * 2.0);
    float tall = (26.0 + 24.0 * pulse * (0.4 + 0.6 * ray)) * ends;
    float v = float(gl_VertexID >> 1);
    float y = b.z + 3.0 * sin(t * 0.2 + s * 0.04) + v * tall;
    v_p = vec3(s, v, ends);
    gl_Position = (ci >= u_n) ? vec4(2.0, 2.0, 2.0, 1.0) : u_vp * vec4(p.x, y, p.y, 1.0);
}
"#;

pub const AURORA_FS: &str = r#"#version 300 es
precision highp float;
uniform float u_time;
uniform float u_strength;
in vec3 v_p;
out vec4 o;
void main() {
    float s = v_p.x;
    float v = clamp(v_p.y, 0.0, 1.0);
    float t = u_time;
    // a bright, soft lower edge and a long fade up into the dark
    float foot = smoothstep(0.0, 0.07, v);
    float fade = pow(1.0 - v, 1.6);
    // the top has no edge of its own: the glow thins to nothing well before the
    // quad ends, so the ragged top of the ribbon never shows
    float top = 1.0 - smoothstep(0.35, 1.0, v);
    float body = foot * (0.2 + 0.8 * fade) * top * top;
    // soft vertical rays, drifting and shimmering along the curtain, more marked
    // higher up where the curtain thins
    float ray = 0.5 + 0.5 * sin(s * 0.85 + sin(s * 0.13 + t * 0.3) * 2.4 - t * 0.55);
    float fine = 0.5 + 0.5 * sin(s * 2.3 + sin(s * 0.31 - t * 0.4) * 3.0 + t * 0.9);
    float rays = mix(0.55, 1.0, ray) * mix(0.8, 1.0, fine);
    rays = mix(rays, pow(rays, 2.0), v);
    // green at the foot, through teal and blue to violet, and red at the very top
    vec3 col = vec3(0.1, 1.0, 0.45);
    col = mix(col, vec3(0.12, 0.92, 0.78), smoothstep(0.08, 0.28, v));
    col = mix(col, vec3(0.2, 0.62, 1.0), smoothstep(0.25, 0.48, v));
    col = mix(col, vec3(0.62, 0.28, 1.0), smoothstep(0.45, 0.72, v));
    col = mix(col, vec3(1.0, 0.2, 0.45), smoothstep(0.75, 1.0, v) * 0.7);
    float a = body * rays * v_p.z * u_strength;
    o = vec4(col * a * 1.25, a);
}
"#;
