pub const CHUNK_VS: &str = r#"#version 300 es
layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec4 a_col;
layout(location = 2) in vec4 a_info;
uniform mat4 u_vp;
uniform float u_time;
uniform vec2 u_wind;
uniform float u_crop;
out vec3 v_wp;
out vec3 v_rest;
out vec3 v_col;
out vec3 v_n;
out float v_ao;
flat out float v_mat;
flat out float v_hold;
flat out float v_edges;
vec3 normalOf(float i) {
    if (i < 0.5) return vec3(1.0, 0.0, 0.0);
    if (i < 1.5) return vec3(-1.0, 0.0, 0.0);
    if (i < 2.5) return vec3(0.0, 1.0, 0.0);
    if (i < 3.5) return vec3(0.0, -1.0, 0.0);
    if (i < 4.5) return vec3(0.0, 0.0, 1.0);
    return vec3(0.0, 0.0, -1.0);
}
SWAY
void main() {
    vec3 p = a_pos;
    float m = floor(a_col.a * 255.0 + 0.5);
    p = sway(p, m);
    if (m == 7.0) {
        p.y -= (1.0 - u_crop) * 0.82;
        m = 6.0;
    }
    v_wp = p;
    v_rest = a_pos;
    v_col = a_col.rgb;
    v_n = normalOf(a_info.x);
    v_ao = a_info.y / 3.0;
    v_hold = a_info.z;
    v_edges = a_info.w;
    v_mat = m;
    gl_Position = u_vp * vec4(p, 1.0);
}
"#;

/// Is this fragment part of a building or tree standing between the eye and
/// whoever the camera is watching? Shared by the scene shader (which drops
/// such fragments) and the mask shader (which marks them, to draw the outline
/// of what was removed). `flags` bit 4 marks building/tree blocks.
pub const OCCLUDE: &str = r#"
uniform vec4 u_focusPts[8];
uniform int u_nFocus;
uniform vec3 u_eye;
uniform float u_clear;
// a steady random number for each half-voxel cell, to dissolve things by
float occHash(vec3 p) {
    p = fract(floor(p * 2.0 + 0.37) * 0.1031);
    p += dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
}
bool occluded(vec3 wp, float flags) {
    // only blocks not resting on the ground (flag 64) are ever taken away, so
    // the bottom row of every wall and trunk stays, wherever it stands
    if (u_clear <= 0.0 || (int(flags + 0.5) & 64) == 0) return false;
    for (int i = 0; i < 8; i++) {
        if (i >= u_nFocus) break;
        vec4 fp = u_focusPts[i];
        if (fp.w < 0.0) {
            // a room laid open like a dolls' house: roof and upper floors gone,
            // and the walls on the camera's side, leaving the far walls and the
            // base of every wall standing
            float hw = -fp.w;
            vec2 d = wp.xz - fp.xz;
            float rel = wp.y - fp.y;
            if (abs(d.x) > hw + 0.6 || abs(d.y) > hw + 0.6 || rel < 0.5) continue;
            // the roof and walls dissolve away a cell at a time as the view eases in
            if (occHash(wp) >= u_clear) continue;
            vec2 eh = normalize(u_eye.xz + vec2(1e-4));
            if (rel > 3.4 || dot(d, eh) > -0.2) return true;
            continue;
        }
        // the line of sight runs through the middle of the person, not their feet
        vec3 dv = wp - (fp.xyz + vec3(0.0, 0.9, 0.0));
        float along = dot(dv, u_eye);
        float side = length(dv - u_eye * along);
        if (along > 0.5 && wp.y > fp.y + 0.5 && side < fp.w * u_clear) return true;
    }
    return false;
}
"#;

/// Marks the fragments `occluded` picks out, for the outline.
pub const MASK_FS: &str = r#"#version 300 es
precision highp float;
in vec3 v_wp;
in vec3 v_rest;
in vec3 v_n;
flat in float v_mat;
flat in float v_hold;
flat in float v_edges;
uniform float u_bare;
OCCLUDE
out vec4 o;
float hash13(vec3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
}
void main() {
    if (!occluded(v_wp, v_edges)) discard;
    // leaves that have fallen aren't drawn, so they leave no outline either
    if (v_mat == 2.0) {
        float h = hash13(floor(v_rest - v_n * 0.01));
        float rank = v_hold > 0.5 ? (v_hold - 1.0) / 254.0 * 0.78 + h * 0.22 : h;
        if (rank < u_bare) discard;
    }
    o = vec4(1.0);
}
"#;

pub const CHUNK_FS: &str = r#"
in vec3 v_wp;
in vec3 v_rest;
in vec3 v_col;
in vec3 v_n;
in float v_ao;
flat in float v_mat;
flat in float v_hold;
flat in float v_edges;
uniform vec3 u_grassTint;
uniform vec3 u_leafTint;
uniform float u_autumn;
uniform float u_bare;
OCCLUDE
uniform float u_blossom;
uniform float u_snow;
uniform float u_wet;
uniform float u_ripe;
uniform float u_winLit;
uniform vec3 u_sky;
uniform float u_rain;
uniform float u_peakSnow;
uniform float u_season;
out vec4 o;
// 0..1: how much a laid-open room's own lamplight reaches this point
float roomGlow(vec3 wp) {
    float g = 0.0;
    for (int i = 0; i < 8; i++) {
        if (i >= u_nFocus) break;
        vec4 fp = u_focusPts[i];
        if (fp.w >= 0.0) continue;
        float hw = -fp.w;
        vec2 d = wp.xz - fp.xz;
        float rel = wp.y - fp.y;
        float inside = (1.0 - smoothstep(hw * 0.5, hw + 0.8, max(abs(d.x), abs(d.y)))) * (1.0 - smoothstep(2.6, 3.8, rel)) * step(-0.6, rel);
        g = max(g, inside);
    }
    return g * u_clear;
}
void main() {
#if FX_MATERIALS
    // how many voxels one pixel spans, for fading out detail that would only
    // alias (taken before any discard, while the derivatives are well defined)
    float fpx = fwidth(v_rest.x) + fwidth(v_rest.y) + fwidth(v_rest.z);
#endif
    vec3 alb = v_col;
    float m = v_mat;
    // buildings and trees between the eye and whoever's watched are removed
    // outright (the post pass draws their outline from the mask)
    if ((int(v_edges + 0.5) & 32) != 0) {
        // a base block's cap shows only where the block above has been taken away
        if (!occluded(v_wp + vec3(0.0, 0.5, 0.0), 80.0)) discard;
    } else if (occluded(v_wp, v_edges)) discard;
    // identify the voxel from its rest position so swaying never changes it
    vec3 cell = floor(v_rest - v_n * 0.01);
    float h = hash13(cell);
    if (m == 3.0 || m == 12.0) {
        alb *= u_grassTint;
    } else if (m == 2.0) {
        if (leafRank(h, v_hold) < u_bare) discard;
        vec3 aut = mix(vec3(0.93, 0.52, 0.14), vec3(0.82, 0.24, 0.12), fract(h * 7.13));
        aut = mix(aut, vec3(0.95, 0.78, 0.22), step(0.72, fract(h * 13.7)));
        alb *= u_leafTint;
        alb = mix(alb, aut, u_autumn * step(0.12, fract(h * 3.3)));
        alb = mix(alb, vec3(0.5, 0.38, 0.24), u_bare * 0.85);
        if (fract(h * 17.31) < u_blossom) alb = vec3(1.0, 0.76, 0.86);
    } else if (m == 9.0) {
        if (u_bare > 0.5) discard;
    } else if (m == 13.0) {
        if (u_season < 0.1 || u_season > 1.05) discard;
    } else if (m == 14.0) {
        if (u_season < 1.0 || u_season > 2.1) discard;
    } else if (m == 6.0) {
        alb = mix(vec3(0.38, 0.66, 0.22), vec3(0.92, 0.76, 0.3), u_ripe) * (0.9 + 0.2 * h);
    } else if (m == 8.0) {
        alb *= mix(vec3(1.0), vec3(0.85, 0.92, 0.88), u_bare);
    }
    // snow lies on the heights through the colder months, everywhere in winter
    float snowAmt = max(u_snow, u_peakSnow * smoothstep(33.0, 42.0, v_wp.y));
    if (snowAmt > 0.0 && v_n.y > 0.5 && m != 4.0 && m != 5.0 && m != 10.0) {
        float drift = vnoise(cell.xz * 0.13) * 0.6 + vnoise(cell.xz * 0.5) * 0.25 + h * 0.15;
        float s = smoothstep(0.0, 0.12, snowAmt * 1.25 - drift);
        alb = mix(alb, vec3(0.93, 0.95, 1.0), s);
    }
#if FX_MATERIALS
    // no two voxels quite alike, and the hard edges of masonry catch a little shade
    if (m == 0.0 || m == 11.0 || m == 3.0 || m == 12.0 || m == 6.0) {
        float k = ((m == 3.0 || m == 12.0) ? 0.2 : 0.14) * (1.0 - smoothstep(1.2, 3.0, fpx));
        alb *= 1.0 + (hash13(cell + 7.0) - 0.5) * k;
    }
    // (a grid one voxel across: from far out it would moire against the pixels)
    if ((m == 0.0 || m == 11.0) && fpx < 1.6) {
        vec3 fr = fract(v_rest);
        vec3 e = min(fr, 1.0 - fr) + abs(v_n);
        float bevel = 1.0 - smoothstep(0.7, 1.6, fpx);
        alb *= 1.0 - 0.1 * bevel * (1.0 - smoothstep(0.0, 0.1, min(e.x, min(e.y, e.z))));
    }
#endif
    vec3 lit = lighting(v_wp, v_n, v_ao);
    vec3 c = alb * lit;
    // inside a room laid open: warm lamplight and firelight, brighter after
    // dark, and a little ambient fill so it never reads as a black box
    float rg = roomGlow(v_wp);
    if (rg > 0.0) {
        vec3 warm = vec3(1.08, 0.82, 0.54);
        float fall = 0.7 + 0.3 * max(v_n.y, 0.0);
        c = mix(c, alb * warm * (0.62 + 0.5 * u_night) * fall + c * 0.35, rg * 0.7);
    }
    if (v_n.y > 0.5) c *= 1.0 - 0.18 * u_wet;
#if FX_MATERIALS
    if (m != 4.0 && m != 5.0 && m != 10.0) {
        // gusts drifting over the fields show as soft patches of light and
        // shade (never straight stripes: from far out those would sweep across
        // the whole view like scan lines)
        if (m == 3.0 || m == 12.0 || m == 6.0) {
            float wl = length(u_wind);
            if (wl > 0.01) {
                vec2 gp = (v_wp.xz - u_wind / wl * u_time * 3.0) * 0.05;
                float g = vnoise(gp) * 0.65 + vnoise(gp * 2.3 + 4.0) * 0.35;
                c *= 1.0 + 0.1 * min(wl, 1.0) * (g - 0.5);
            }
        }
        // wet surfaces take a sheen from the sun
        if (u_wet > 0.05 && u_sunDir.y > 0.0) {
            float sp = pow(max(dot(reflect(-u_sunDir, v_n), u_eye), 0.0), 18.0) * u_wet * (v_n.y > 0.5 ? 1.0 : 0.4);
            if (sp > 0.01) c += u_sunCol * sp * 0.35 * cloudShade(v_wp) * shadowAt(v_wp, v_n);
        }
        // snow glitters in the sun
        if (snowAmt > 0.2 && v_n.y > 0.5 && u_sunDir.y > 0.1) {
            float gl = step(0.9965, hash13(vec3(floor(v_wp.xz * 3.0), floor(u_time * 1.5 + h * 9.0))));
            c += vec3(1.4) * gl * snowAmt * (1.0 - u_night) * cloudShade(v_wp);
        }
    }
#endif
    // puddles on paths and cobbles: still water reflecting the sky, a sun
    // glint, and rings where raindrops land
    if (m == 11.0 && u_wet > 0.05 && v_n.y > 0.5) {
        float pud = smoothstep(0.5, 0.58, vnoise(v_wp.xz * 0.33 + 17.0) + vnoise(v_wp.xz * 1.1) * 0.15 - (1.0 - u_wet) * 0.35);
        vec3 sky = u_sky * (0.75 + 0.25 * vnoise(v_wp.xz * 0.5 + u_time * 0.05));
        float glint = pow(max(dot(reflect(-u_sunDir, vec3(0.0, 1.0, 0.0)), normalize(vec3(-0.6, 0.65, -0.45))), 0.0), 40.0);
        float rings = u_rain * step(0.93, vnoise(v_wp.xz * 4.0 + floor(u_time * 6.0) * 1.7));
        vec3 wet = sky * 0.85 + u_sunCol * glint * 0.8 + rings * 0.12;
        c = mix(c, wet, pud * 0.75);
    }
    if (m == 4.0) {
        float on = step(h, u_winLit);
        vec3 glass = alb * lit * 0.5 + u_ambSky * 0.1;
        c = mix(glass, vec3(1.0, 0.78, 0.42) * 1.2, on * u_night);
    } else if (m == 5.0) {
        c = mix(alb * lit, vec3(1.0, 0.86, 0.55) * 1.5, max(u_lamp, 0.12));
    } else if (m == 10.0) {
        c = v_col * (1.25 + 0.3 * sin(u_time * 9.0 + h * 20.0));
    }
    o = vec4(c, 1.0);
}
"#;

pub const WATER_VS: &str = r#"#version 300 es
layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec4 a_col;
layout(location = 2) in vec4 a_info;
uniform mat4 u_vp;
uniform float u_time;
uniform float u_tide;
out vec3 v_wp;
void main() {
    vec3 p = a_pos;
    if (a_info.x > 1.5 && a_info.x < 2.5) {
        // (a fountain's or a pool's water is flagged still: the tide doesn't lift it)
        // three swells running different ways, so the surface heaves rather than just bobs
        float swell = sin(u_time * 1.3 + p.x * 0.7 + p.z * 0.5) * 0.022
            + sin(u_time * 1.9 - p.x * 0.45 + p.z * 0.8) * 0.016
            + sin(u_time * 0.9 + p.x * 1.1 - p.z * 0.3) * 0.01;
        p.y += swell - 0.03 + (a_info.z > 0.5 ? 0.0 : u_tide);
    }
    v_wp = p;
    gl_Position = u_vp * vec4(p, 1.0);
}
"#;

pub const WATER_FS: &str = r#"
in vec3 v_wp;
uniform float u_ice;
uniform vec3 u_eye;
uniform vec3 u_sky;
#if FX_WATER >= 2
// the scene as it stood before the water: its colour and depth, for the
// depth of the water, shore foam and reflections of the banks
uniform sampler2D u_sceneColor;
uniform sampler2D u_sceneDepth;
uniform vec2 u_invSize;
uniform float u_depthRange;
uniform mat4 u_vp;
#endif
out vec4 o;
#if FX_WATER >= 3
float caustic(vec2 p, float t) {
    float a = vnoise(p * 1.1 + vec2(t * 0.35, t * 0.2));
    float b = vnoise(p * 1.9 - vec2(t * 0.3, -t * 0.25) + 7.0);
    float l = abs(a - b);
    return pow(1.0 - smoothstep(0.0, 0.12, l), 2.0);
}
#endif
#if FX_WATER >= 1
// the ripple height: layers drifting different ways over a slowly swirling
// surface, so the ripples shift and cross instead of sliding off together
float ripple(vec2 p, float t) {
    vec2 sw = vec2(vnoise(p * 0.18 + vec2(t * 0.05, 0.0)), vnoise(p * 0.18 + vec2(7.0, -t * 0.04))) - 0.5;
    p += sw * 2.2;
    return vnoise(p * 0.5 + vec2(t * 0.25, t * 0.18)) * 0.45
         + vnoise(p * 1.1 - vec2(t * 0.4, -t * 0.3)) * 0.35
         + vnoise(p * 2.4 + vec2(-t * 0.65, t * 0.5)) * 0.2;
}
#endif
void main() {
    vec2 q = v_wp.xz;
    float w1 = vnoise(q * 0.35 + vec2(u_time * 0.25, u_time * 0.18));
    float w2 = vnoise(q * 0.9 - vec2(u_time * 0.4, -u_time * 0.3));
    float w = w1 * 0.6 + w2 * 0.4;
    vec3 alb = mix(vec3(0.12, 0.38, 0.56), vec3(0.3, 0.6, 0.76), w);
    float sh = shadowAt(v_wp, vec3(0.0, 1.0, 0.0)) * cloudShade(v_wp);
    vec3 lit = mix(u_ambGround, u_ambSky, 0.85) + u_sunCol * max(u_sunDir.y, 0.0) * sh * 0.8;
    vec3 c = alb * lit;
    float a = 0.74;
    float vdepth = 9.0;
    float foam = 0.0;
#if FX_WATER >= 2
    vec2 suv = gl_FragCoord.xy * u_invSize;
    float fd = texture(u_sceneDepth, suv).r;
    // how far down the bed is, from how much further off it is along the view
    vdepth = max(fd - gl_FragCoord.z, 0.0) * u_depthRange * u_eye.y;
    float deepK = 1.0 - exp(-vdepth * 0.3);
    c *= mix(1.15, 0.85, deepK);
    a = mix(0.4, 0.8, deepK);
    // foam lapping at the shore
    float edge = 1.0 - smoothstep(0.0, 0.5, vdepth);
    float lap = sin(vdepth * 16.0 - u_time * 1.6 + w1 * 5.0) * 0.5 + 0.5;
    foam = edge * (0.45 + 0.55 * smoothstep(0.35, 0.8, lap * (0.6 + 0.4 * w2)));
    c = mix(c, vec3(0.92, 0.96, 1.0) * min(lit * 0.8 + 0.2, vec3(1.2)), foam * 0.75);
    a = max(a, foam * 0.92);
#endif
#if FX_WATER >= 3
    c += u_sunCol * caustic(q, u_time) * sh * (1.0 - smoothstep(0.2, 2.2, vdepth)) * 0.4 * step(0.0, u_sunDir.y);
#endif
    float sp = step(0.84, w2) * step(0.55, w1);
    c += u_sunCol * sp * 0.5 * sh;
    c += lamps(v_wp, vec3(0.0, 1.0, 0.0)) * 0.35;
#if FX_WATER >= 1
    // the sky (and, further up, the banks) mirrored in the ripples
    vec2 ex = vec2(0.1, 0.0);
    float rh = ripple(q, u_time);
    float rhx = ripple(q + ex.xy, u_time);
    float rhz = ripple(q + ex.yx, u_time);
    vec3 nw = normalize(vec3((rh - rhx) * 1.6, 1.0, (rh - rhz) * 1.6));
    float fr = 0.14 + 0.5 * pow(1.0 - max(dot(nw, u_eye), 0.0), 2.5);
    vec3 rd = reflect(-u_eye, nw);
    vec3 refl = mix(u_sky, u_sky * vec3(0.6, 0.78, 1.1), pow(clamp(rd.y, 0.0, 1.0), 0.6));
    float sunD = max(dot(rd, u_sunDir), 0.0);
    refl += u_sunCol * (pow(sunD, 90.0) * 1.6 + pow(sunD, 700.0) * 5.0) * sh;
#if FX_WATER >= 3
    float jit = hash12(gl_FragCoord.xy);
    for (int i = 0; i < 14; i++) {
        vec3 sp3 = v_wp + rd * ((float(i) + jit + 0.5) * 1.6);
        vec4 cp = u_vp * vec4(sp3, 1.0);
        vec3 sc = cp.xyz / cp.w * 0.5 + 0.5;
        if (sc.x < 0.0 || sc.x > 1.0 || sc.y < 0.0 || sc.y > 1.0) break;
        // the ray has gone behind something the camera sees there
        float dz = (sc.z - texture(u_sceneDepth, sc.xy).r) * u_depthRange;
        if (dz > 0.0 && dz < 2.5) {
            refl = texture(u_sceneColor, sc.xy).rgb;
            break;
        }
    }
#endif
    c = mix(c, refl, fr);
    a = mix(a, 0.95, fr * 0.5);
#endif
    // starlight on still water
    if (u_night > 0.4) {
        float hs = hash12(floor(v_wp.xz * 2.5));
        float tw = step(0.992, hs) * (0.5 + 0.5 * sin(u_time * 3.0 + hs * 40.0));
        c += vec3(0.9, 0.95, 1.0) * tw * u_night * (1.0 - u_cloud) * 0.9;
    }
    if (u_ice > 0.0) {
        vec3 ice = vec3(0.8, 0.88, 0.95) * lit;
        c = mix(c, ice, u_ice);
        a = mix(a, 0.95, u_ice);
    }
    o = vec4(c, a);
}
"#;
