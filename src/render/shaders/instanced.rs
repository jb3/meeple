pub const INST_VS: &str = r#"#version 300 es
layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec3 a_nrm;
layout(location = 2) in vec4 i_a;
layout(location = 3) in vec4 i_b;
layout(location = 4) in vec4 i_c;
uniform mat4 u_vp;
out vec3 v_wp;
out vec3 v_n;
out vec3 v_col;
flat out float v_mat;
flat out float v_alpha;
void main() {
    float c = cos(i_a.w);
    float s = sin(i_a.w);
    vec3 lp = a_pos * i_b.xyz;
    vec3 rp = vec3(lp.x * c + lp.z * s, lp.y, -lp.x * s + lp.z * c);
    v_n = vec3(a_nrm.x * c + a_nrm.z * s, a_nrm.y, -a_nrm.x * s + a_nrm.z * c);
    v_wp = i_a.xyz + rp;
    v_col = i_c.rgb;
    v_mat = i_b.w;
    v_alpha = i_c.w;
    gl_Position = u_vp * vec4(v_wp, 1.0);
}
"#;

pub const INST_FS: &str = r#"
in vec3 v_wp;
in vec3 v_n;
in vec3 v_col;
flat in float v_mat;
flat in float v_alpha;
out vec4 o;
void main() {
    if (v_alpha < 0.999 && bayer4(gl_FragCoord.xy) >= v_alpha) discard;
    vec3 c;
    if (v_mat > 11.5 && v_mat < 12.5) {
        c = v_col;
    } else if (v_mat > 9.5 && v_mat < 10.5) {
        c = v_col * 1.5;
    } else {
        c = v_col * lighting(v_wp, v_n, 1.0);
        if (v_mat > 12.5) c = mix(c, v_col * 1.25, u_night);
    }
    o = vec4(c, 1.0);
}
"#;

pub const SHADOW_VS: &str = r#"#version 300 es
layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec4 a_col;
layout(location = 2) in vec4 a_info;
uniform mat4 u_vp;
uniform float u_time;
uniform vec2 u_wind;
out vec3 v_wp;
flat out float v_mat;
flat out float v_nidx;
flat out float v_hold;
flat out float v_edges;
SWAY
void main() {
    v_wp = a_pos;
    v_mat = floor(a_col.a * 255.0 + 0.5);
    v_nidx = a_info.x;
    v_hold = a_info.z;
    v_edges = a_info.w;
    gl_Position = u_vp * vec4(sway(a_pos, v_mat), 1.0);
}
"#;

pub const SHADOW_FS: &str = r#"#version 300 es
precision highp float;
in vec3 v_wp;
flat in float v_mat;
flat in float v_nidx;
flat in float v_hold;
flat in float v_edges;
uniform float u_bare;
uniform float u_season;
OCCLUDE
out vec4 o;
float hash13(vec3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
}
// When the leaves fall, those against the trunk (hold near 1) go last; hold 0
// means no preference, so the voxel's own hash decides.
float leafRank(float h, float hold) {
    return hold > 0.5 ? (hold - 1.0) / 254.0 * 0.78 + h * 0.22 : h;
}
void main() {
    // taken away from the view, taken out of the shadows too
    if (occluded(v_wp, v_edges)) discard;
    if ((int(v_edges + 0.5) & 32) != 0) discard;
    if (v_mat == 2.0 || v_mat == 9.0) {
        vec3 n = v_nidx < 0.5 ? vec3(1,0,0) : v_nidx < 1.5 ? vec3(-1,0,0) : v_nidx < 2.5 ? vec3(0,1,0)
               : v_nidx < 3.5 ? vec3(0,-1,0) : v_nidx < 4.5 ? vec3(0,0,1) : vec3(0,0,-1);
        float h = hash13(floor(v_wp - n * 0.01));
        if (v_mat == 2.0 && leafRank(h, v_hold) < u_bare) discard;
        if (v_mat == 9.0 && u_bare > 0.5) discard;
    }
    if (v_mat == 13.0 && (u_season < 0.1 || u_season > 1.05)) discard;
    if (v_mat == 14.0 && (u_season < 1.0 || u_season > 2.1)) discard;
    o = vec4(1.0);
}
"#;

pub const SHADOW_INST_VS: &str = r#"#version 300 es
layout(location = 0) in vec3 a_pos;
layout(location = 2) in vec4 i_a;
layout(location = 3) in vec4 i_b;
layout(location = 4) in vec4 i_c;
uniform mat4 u_vp;
flat out float v_alpha;
void main() {
    v_alpha = i_c.w;
    float c = cos(i_a.w);
    float s = sin(i_a.w);
    vec3 lp = a_pos * i_b.xyz;
    vec3 rp = vec3(lp.x * c + lp.z * s, lp.y, -lp.x * s + lp.z * c);
    gl_Position = u_vp * vec4(i_a.xyz + rp, 1.0);
    // hair-thin boxes (fishing lines) are narrower than a shadow texel and
    // flicker in and out as the map shifts, so they cast nothing
    if (min(i_b.x, min(i_b.y, i_b.z)) < 0.03) gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
}
"#;

/// Shadows of fading boxes dissolve with the same dither as the boxes.
pub const SHADOW_INST_FS: &str = r#"#version 300 es
precision mediump float;
flat in float v_alpha;
out vec4 o;
BAYER
void main() {
    if (v_alpha < 0.999 && bayer4(gl_FragCoord.xy) >= v_alpha) discard;
    o = vec4(1.0);
}
"#;
