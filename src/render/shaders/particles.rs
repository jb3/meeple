/// Rain, snow and falling leaves, worked out from nothing but the instance
/// number and the clock: no buffers, no per-frame uploads.
pub const PART_VS: &str = r#"#version 300 es
precision highp float;
uniform mat4 u_vp;
uniform float u_time;
uniform vec2 u_center;
uniform float u_ext;
uniform float u_ground;
uniform vec2 u_wind;
uniform int u_kind;
uniform float u_texel;
uniform vec3 u_right;
uniform vec3 u_up;
uniform float u_br;
uniform float u_rain;
out vec4 v_col;
uint pcg(uint v) {
    uint s = v * 747796405u + 2891336453u;
    uint w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
    return (w >> 22u) ^ w;
}
float rnd(uint v) {
    return float(pcg(v) & 0xffffffu) / 16777216.0;
}
float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash12(i), hash12(i + vec2(1.0, 0.0)), f.x), mix(hash12(i + vec2(0.0, 1.0)), hash12(i + vec2(1.0, 1.0)), f.x), f.y);
}
void main() {
    uint id = uint(gl_InstanceID) * 6u + 11u;
    float r1 = rnd(id);
    float r2 = rnd(id + 1u);
    float r3 = rnd(id + 2u);
    float r4 = rnd(id + 3u);
    float r5 = rnd(id + 4u);
    bool snow = u_kind == 1;
    bool leaf = u_kind == 2;
    float H = leaf ? 20.0 : 44.0;
    float fall = (snow ? 2.2 : (leaf ? 0.9 : 26.0)) * (0.85 + 0.3 * r4);
    float fy = mod(r3 * H + u_time * fall, H);
    float y = u_ground + H - 6.0 - fy;
    vec2 drift = (leaf ? (u_wind + vec2(0.3, 0.2)) * 1.2 : u_wind * (snow ? 0.9 : 0.25)) * (H - fy);
    vec2 span = vec2(2.0 * u_ext);
    vec2 lo = u_center - u_ext;
    vec2 p = vec2(r1, r2) * span + drift;
    if (snow) p += vec2(sin(u_time * 0.8 + r5 * 40.0), cos(u_time * 0.7 + r5 * 55.0)) * 0.6;
    if (leaf) p += vec2(sin(u_time * 1.3 + r5 * 40.0) * 0.7, 0.0);
    // the drops stay where they are in the world; the box just follows the camera
    p = mod(p - lo, span) + lo;
    float keep = 1.0;
    if (!leaf) {
        // rain and snow come in curtains that drift with the wind
        float n = vnoise((p - u_wind * u_time * 3.0) / 160.0) * 1.2 + u_rain * 0.6 - 0.4;
        keep = 0.2 + 0.8 * smoothstep(-0.25, 0.3, n);
    }
    vec2 edge = min(p - lo, lo + span - p);
    float fade = smoothstep(0.0, 6.0, min(edge.x, edge.y));
    vec2 cc = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1)) * 2.0 - 1.0;
    vec3 pos = vec3(p.x, y, p.y);
    vec3 col;
    float alpha = fade;
    if (snow) {
        float s = max(u_texel * 0.55, 0.05);
        pos += u_right * cc.x * s + u_up * cc.y * s;
        col = vec3(0.95, 0.97, 1.0);
    } else if (leaf) {
        float a = u_time * 2.0 + r5 * 30.0;
        vec2 rc = vec2(cc.x * cos(a) - cc.y * sin(a), cc.x * sin(a) + cc.y * cos(a));
        pos += u_right * rc.x * 0.09 + u_up * rc.y * 0.07;
        col = r4 < 0.33 ? vec3(0.9, 0.5, 0.15) : (r4 < 0.66 ? vec3(0.8, 0.25, 0.1) : vec3(0.95, 0.75, 0.2));
    } else {
        float w = max(u_texel * 0.5, 0.03);
        pos += u_right * cc.x * w + vec3(0.0, 1.0, 0.0) * cc.y * 0.4;
        col = vec3(0.69, 0.73, 0.79);
        alpha *= 0.85;
    }
    v_col = vec4(col * u_br, alpha);
    gl_Position = (r5 > keep) ? vec4(2.0, 2.0, 2.0, 1.0) : u_vp * vec4(pos, 1.0);
}
"#;

pub const PART_FS: &str = r#"#version 300 es
precision mediump float;
in vec4 v_col;
out vec4 o;
void main() {
    o = v_col;
}
"#;
