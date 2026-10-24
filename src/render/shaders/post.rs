pub const POST_VS: &str = r#"#version 300 es
out vec2 v_uv;
void main() {
    vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    v_uv = p;
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub const POST_FS: &str = r#"#version 300 es
precision highp float;
precision highp sampler3D;
precision highp sampler2DShadow;
in vec2 v_uv;
uniform sampler2D u_color;
uniform sampler2D u_depth;
uniform sampler2D u_mask;
uniform float u_maskOn;
uniform vec2 u_texel;
uniform mat4 u_invVP;
uniform vec3 u_fogCol;
uniform float u_fog;
uniform float u_flash;
uniform float u_outline;
uniform float u_rain;
uniform float u_time;
uniform vec2 u_shift;
uniform vec2 u_wind;
uniform float u_rainbow;
uniform vec3 u_bowA;
uniform vec2 u_bowN;
uniform float u_bowR;
uniform float u_aurora;
uniform vec4 u_meteor;
uniform float u_meteorT;
uniform float u_tilt;
uniform float u_hdr;
uniform float u_night;
uniform vec3 u_eye;
uniform vec3 u_camR;
uniform vec3 u_camU;
uniform vec3 u_sunDir;
uniform vec3 u_sunCol;
uniform float u_cloud;
uniform float u_moon;
uniform float u_depthRange;
#if FX_MIPBLOOM
uniform sampler2D u_bloom;
uniform sampler2D u_bloomWide;
#endif
#if FX_GRADE
uniform sampler3D u_lut;
#endif
#if FX_DOF
uniform float u_dofR;
uniform float u_focusZ;
uniform float u_dofSpan;
#endif
#if FX_SSAO
uniform float u_aoRad;
uniform float u_aoK;
uniform float u_aoMin;
#endif
#if FX_RAYS
uniform highp sampler2DShadow u_shadowP;
uniform mat4 u_lightVP;
uniform float u_rayLen;
#endif
out vec4 o;
float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
// Interleaved gradient noise: an even, lattice-free pattern per pixel, for
// turning sample patterns without banding.
float ign(vec2 p) {
    return fract(52.9829189 * fract(dot(p, vec2(0.06711056, 0.00583715))));
}
float vnoise2(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash12(i), hash12(i + vec2(1.0, 0.0)), f.x), mix(hash12(i + vec2(0.0, 1.0)), hash12(i + vec2(1.0, 1.0)), f.x), f.y);
}
// Eases values above `t` towards 1 instead of clipping them.
vec3 tonemap(vec3 x) {
    const float t = 0.82;
    vec3 hi = max(x - t, 0.0);
    return min(x, vec3(t)) + (1.0 - t) * (1.0 - exp(-hi / (1.0 - t)));
}
#if !FX_MIPBLOOM
// A soft halo from whatever's brighter than white: lamps, fire, windows at
// night, sun glints. Three rings of taps, turned by a little noise.
vec3 bloom(vec2 uv, float aspect) {
    vec3 acc = vec3(0.0);
    float a0 = hash12(gl_FragCoord.xy) * 6.2832;
    for (int r = 0; r < 3; r++) {
        float rad = r == 0 ? 0.006 : r == 1 ? 0.016 : 0.034;
        float w = r == 0 ? 0.5 : r == 1 ? 0.3 : 0.2;
        for (int k = 0; k < 6; k++) {
            float a = a0 + float(k) * 1.0472 + float(r);
            vec2 off = vec2(cos(a) / aspect, sin(a)) * rad;
            acc += max(texture(u_color, uv + off).rgb - 0.9, 0.0) * w;
        }
    }
    return acc / 6.0;
}
#endif
// The camera looks straight along one direction, so a view direction for the
// sky is invented from the screen position: tilted up towards the top of the
// picture, as if the view were wide. It turns with the camera.
vec3 skyDir(vec2 ndc, float aspect) {
    return normalize(-u_eye * 0.9 + u_camR * ndc.x * 0.8 * aspect + u_camU * (ndc.y * 0.8 + 0.8));
}
// Northern lights: curtains hanging in the sky, fixed to compass directions
// (so they swing across the view as the camera turns), with a bright lower
// edge, vertical rays and slow pulses.
// One curtain: a hard, bright lower edge with a long glow above it, pleated
// into vertical rays that drift and pulse.
vec3 auroraCurtain(float az, float el, vec2 ap, float t, float lift, float seed) {
    float base = 0.1 + lift + 0.13 * sin(az * 2.0 + t * 0.07 + seed) + 0.07 * sin(az * 5.0 - t * 0.11 + seed * 2.0);
    float h = el - base;
    float curtain = smoothstep(-0.03, 0.012, h) * exp(-max(h, 0.0) * 3.0);
    float rays = 0.5 + 0.5 * sin(az * 38.0 + vnoise2(ap * 3.0 + t * 0.05 + seed) * 8.0 + t * 0.3);
    rays = mix(0.45, 1.0, rays) * (0.55 + 0.9 * vnoise2(vec2(az * 14.0 - t * 0.08 + seed, el * 3.0)));
    float span = smoothstep(0.05, 0.35, vnoise2(ap * 2.2 + vec2(t * 0.02, 3.0 + seed)));
    float pulse = 0.7 + 0.3 * sin(t * 0.5 + az * 3.0 + seed);
    // green at the bright foot, through teal and violet, to red at the top
    vec3 col = mix(vec3(0.1, 1.0, 0.45), vec3(0.2, 0.75, 1.0), smoothstep(0.0, 0.18, h));
    col = mix(col, vec3(0.7, 0.3, 1.0), smoothstep(0.15, 0.5, h));
    col = mix(col, vec3(1.0, 0.2, 0.4), smoothstep(0.4, 0.8, h) * 0.6);
    return col * curtain * rays * pulse * span;
}
// Northern lights: layered curtains hanging in the sky, fixed to compass
// directions (so they swing across the view as the camera turns).
vec3 auroraAt(vec3 sd) {
    float az = atan(sd.x, sd.z);
    float el = asin(clamp(sd.y, -1.0, 1.0));
    vec2 ap = vec2(cos(az), sin(az));
    float t = u_time;
    vec3 a = auroraCurtain(az, el, ap, t, 0.0, 0.0);
    vec3 b = auroraCurtain(az + 0.9, el, ap, t * 0.8, 0.18, 4.0);
    vec3 c = auroraCurtain(az - 1.7, el, ap, t * 1.1, 0.06, 9.0);
    return (a + 0.8 * b + 0.6 * c) * 1.7;
}
#if FX_SKY
// The sky behind the far edge of the world.
vec3 skyColor(vec2 ndc, vec3 horizon, float aspect) {
    vec3 sd = skyDir(ndc, aspect);
    vec3 c = mix(horizon, horizon * vec3(0.55, 0.72, 1.05), pow(max(sd.y, 0.0), 0.7));
    float ds = dot(sd, u_sunDir);
    // the sun by day, a phase of moon by night
    vec3 t1 = normalize(cross(u_sunDir, vec3(0.0, 1.0, 0.0)));
    vec3 t2 = cross(u_sunDir, t1);
    float rad = mix(0.022, 0.03, u_night);
    vec2 lc = vec2(dot(sd, t1), dot(sd, t2)) / rad;
    float r2 = dot(lc, lc);
    if (r2 < 1.0) {
        vec3 disc = mix(vec3(1.0, 0.95, 0.8) * 2.2, vec3(0.88, 0.92, 1.0) * 0.95, u_night);
        float ph = u_moon * 6.2832;
        vec3 sn = vec3(lc, sqrt(1.0 - r2));
        float lit = smoothstep(-0.04, 0.06, dot(sn, vec3(sin(ph), 0.0, -cos(ph))));
        disc *= mix(1.0, mix(0.06, 1.0, lit), u_night);
        c = mix(c, disc, smoothstep(1.0, 0.85, r2) * (1.0 - u_cloud * 0.7));
    }
    c += mix(u_sunCol, vec3(0.2, 0.25, 0.35), u_night) * (pow(max(ds, 0.0), 40.0) * 0.3 + pow(max(ds, 0.0), 6.0) * 0.07) * (1.0 - u_cloud * 0.6);
#if FX_SKY >= 2
    float nt = smoothstep(0.35, 0.8, u_night) * (1.0 - u_cloud);
    if (nt > 0.01) {
        vec2 sp = sd.xz / (sd.y + 1.3) * 140.0;
        float h = hash12(floor(sp));
        float star = step(0.988, h) * (0.6 + 0.4 * sin(u_time * 3.0 + h * 80.0));
        c += vec3(0.9, 0.95, 1.0) * star * nt * smoothstep(0.0, 0.2, sd.y);
    }
    vec2 cp = sd.xz / (max(sd.y, 0.0) + 0.22) * 0.9 + u_wind * u_time * 0.02;
    float n = vnoise2(cp * 1.3) * 0.55 + vnoise2(cp * 3.1 + 5.0) * 0.3 + vnoise2(cp * 7.0 + 9.0) * 0.15;
    float cov = smoothstep(1.0 - u_cloud - 0.05, 1.0 - u_cloud + 0.25, n) * smoothstep(0.0, 0.18, sd.y);
    vec3 cloudCol = horizon * 0.85 + u_sunCol * 0.35 * (0.5 + 0.5 * clamp(ds + 0.4, 0.0, 1.0));
    c = mix(c, cloudCol, cov * 0.85);
#endif
    return c;
}
#endif
void main() {
    float aspect = u_texel.y / u_texel.x;
    vec2 uv = v_uv - u_shift;
    vec3 c = texture(u_color, uv).rgb;
#if FX_CAB
    {
        // the faintest colour fringe towards the corners
        vec2 dir = v_uv - 0.5;
        float k = dot(dir, dir) * 0.008;
        c.r = texture(u_color, uv - dir * k).r;
        c.b = texture(u_color, uv + dir * k).b;
    }
#endif
    // photograph mode: blur towards the top and bottom, like a miniature
    if (u_tilt > 0.01) {
        float k = u_tilt * smoothstep(0.1, 0.5, abs(v_uv.y - 0.5)) * 4.0;
        vec3 acc = vec3(0.0);
        for (int i = -2; i <= 2; i++) {
            for (int j = -2; j <= 2; j++) {
                acc += texture(u_color, uv + vec2(float(i), float(j)) * k * u_texel).rgb;
            }
        }
        c = mix(c, acc / 25.0, smoothstep(0.1, 0.5, abs(v_uv.y - 0.5)));
    }
#if FX_DOF
    // a shallow depth of field in the ordinary view, focused on the camera's
    // target: what's nearer or further goes soft
    if (u_tilt <= 0.01 && u_dofR > 0.01) {
        float dd = texture(u_depth, uv).r;
        float coc = dd < 1.0 ? clamp(abs(dd - u_focusZ) * u_depthRange / u_dofSpan - 0.3, 0.0, 1.0) : 0.0;
        if (coc > 0.02) {
            vec3 acc = vec3(0.0);
            float rot = ign(gl_FragCoord.xy) * 6.2832;
            for (int i = 0; i < 12; i++) {
                float a = rot + float(i) * 2.39996;
                float r = sqrt((float(i) + 0.5) / 12.0) * coc * u_dofR;
                acc += texture(u_color, uv + vec2(cos(a), sin(a)) * r * u_texel).rgb;
            }
            c = mix(c, acc / 12.0, smoothstep(0.02, 0.25, coc));
        }
    }
#endif
    if (u_hdr > 0.5) {
#if FX_MIPBLOOM
        vec3 glow = texture(u_bloom, uv).rgb * 0.6;
        vec3 wide = texture(u_bloomWide, uv).rgb;
        // lit windows and lamps spill their light onto whatever's near
        c += wide * c * u_night * 1.4;
        // a faint horizontal streak off the brightest lights
        vec3 streak = vec3(0.0);
        for (int i = -3; i <= 3; i++) {
            streak += texture(u_bloomWide, uv + vec2(float(i) * 0.02, 0.0)).rgb * (1.0 - abs(float(i)) / 4.0);
        }
        glow += streak * 0.05;
        // and a speck of lens dirt catching it
        float dirt = vnoise2(v_uv * vec2(7.0 * aspect, 7.0)) * 0.6 + vnoise2(v_uv * vec2(23.0 * aspect, 23.0)) * 0.4;
        glow += wide * pow(dirt, 3.0) * 0.5;
        c = tonemap(c + glow);
#else
        c = tonemap(c + bloom(uv, aspect) * 0.9);
#endif
    }
    // the outline of whatever's been seen through: where the mask meets what
    // isn't masked
    if (u_maskOn > 0.5) {
        float m0 = texture(u_mask, uv).r;
        vec2 o1 = u_texel * 2.0;
        float mn = min(min(texture(u_mask, uv + vec2(o1.x, 0.0)).r, texture(u_mask, uv - vec2(o1.x, 0.0)).r),
                       min(texture(u_mask, uv + vec2(0.0, o1.y)).r, texture(u_mask, uv - vec2(0.0, o1.y)).r));
        if (m0 > 0.5 && mn < 0.5) c = mix(c, c * 0.3, 0.9);
    }
    float d = texture(u_depth, uv).r;
    float dl = texture(u_depth, uv - vec2(u_texel.x, 0.0)).r;
    float dr = texture(u_depth, uv + vec2(u_texel.x, 0.0)).r;
    float du = texture(u_depth, uv + vec2(0.0, u_texel.y)).r;
    float dd = texture(u_depth, uv - vec2(0.0, u_texel.y)).r;
    float lap = (dl + dr + du + dd) - 4.0 * d;
    c *= 1.0 - u_outline * smoothstep(0.0005, 0.0016, lap);
#if FX_SSAO
    if (d < 1.0) {
        // ambient occlusion from the depth alone: fit the plane the pixel lies
        // in (taking the flatter side at an edge), then count the neighbours
        // that stand in front of it
        vec2 sl = vec2(abs(d - dl) < abs(dr - d) ? d - dl : dr - d, abs(d - dd) < abs(du - d) ? d - dd : du - d);
        float occ = 0.0;
        // one pattern per rendered texel (not per screen pixel), so the result
        // stays chunky and doesn't crawl as the picture moves
        float rot = ign(floor(uv / u_texel)) * 6.2832;
        for (int i = 0; i < FX_SSAO_TAPS; i++) {
            float a = rot + float(i) * 2.39996;
            float r = sqrt((float(i) + 0.5) / float(FX_SSAO_TAPS)) * u_aoRad;
            vec2 off = vec2(cos(a), sin(a)) * r;
            float expect = d + dot(sl, off);
            float ds = texture(u_depth, uv + off * u_texel).r;
            float hgt = (expect - ds) * u_depthRange;
            // in front of the plane, but not so far in front that it's another object
            // (bumps smaller than a texel or a tuft of grass don't count)
            occ += smoothstep(u_aoMin, u_aoMin + 0.4, hgt) * (1.0 - smoothstep(1.2, 3.0, hgt));
        }
        c *= 1.0 - u_aoK * occ / float(FX_SSAO_TAPS);
    }
#endif
    vec3 fogC = u_fogCol;
#if FX_FOGSCATTER
    // looking towards the sun, haze glows
    fogC += u_sunCol * pow(max(dot(-u_eye, u_sunDir), 0.0), 2.0) * 0.28 * smoothstep(0.0, 0.25, u_sunDir.y) * (1.0 - u_night);
#endif
    if (d < 1.0) {
        vec4 wp = u_invVP * vec4(uv * 2.0 - 1.0, d * 2.0 - 1.0, 1.0);
        wp /= wp.w;
        float hf = exp(-max(wp.y - 12.0, 0.0) / 8.0);
        // mist rolls through in drifting banks, pooling in the valleys
        float roll = 0.5 + vnoise2(wp.xz / 45.0 + u_wind * u_time * 0.02 + vec2(u_time * 0.004, 0.0));
        float f = clamp(u_fog * (0.2 + 0.8 * hf) * roll, 0.0, 0.9);
        c = mix(c, fogC, f);
#if FX_RAYS
        // shafts of light: march from the surface back towards the eye through
        // the shadow map, adding sunlight scattered by haze wherever it's lit
        if (u_sunDir.y > 0.03) {
            float vis = 0.0;
            float jit = fract(ign(gl_FragCoord.xy) + u_time * 0.618);
            for (int i = 0; i < FX_RAYS_STEPS; i++) {
                vec3 sp = wp.xyz + u_eye * ((float(i) + jit) / float(FX_RAYS_STEPS) * u_rayLen);
                vec4 lp = u_lightVP * vec4(sp, 1.0);
                vec3 q = lp.xyz / lp.w * 0.5 + 0.5;
                float inb = step(0.0, q.x) * step(q.x, 1.0) * step(0.0, q.y) * step(q.y, 1.0) * step(q.z, 1.0);
                vis += mix(1.0, texture(u_shadowP, vec3(q.xy, q.z - 0.001)), inb);
            }
            vis /= float(FX_RAYS_STEPS);
            float low = 1.0 - smoothstep(0.1, 0.6, u_sunDir.y);
            float dens = min((0.04 + u_fog * 0.5 + low * 0.14) * (1.0 - u_cloud * 0.6), 0.25);
            float phase = 0.6 + 1.4 * pow(max(dot(-u_eye, u_sunDir), 0.0), 3.0);
            c += u_sunCol * vis * dens * phase * 0.5;
        }
#endif
    } else {
#if FX_SKY
        c = skyColor(uv * 2.0 - 1.0, fogC, aspect);
#else
        c = fogC;
#endif
    }
    float l = dot(c, vec3(0.3, 0.59, 0.11));
    c = mix(c, vec3(l), u_rain * 0.25);
    // a rainbow standing in the world: an arc in a vertical plane (u_bowA,
    // u_bowN), found where each view ray crosses that plane, and hidden by
    // anything nearer than the plane
    if (u_rainbow > 0.01) {
        vec2 ndc = uv * 2.0 - 1.0;
        vec4 pa = u_invVP * vec4(ndc, -1.0, 1.0);
        vec4 pb = u_invVP * vec4(ndc, 1.0, 1.0);
        vec3 ro = pa.xyz / pa.w;
        vec3 rd = normalize(pb.xyz / pb.w - ro);
        vec3 nrm = vec3(u_bowN.x, 0.0, u_bowN.y);
        float den = dot(rd, nrm);
        if (abs(den) > 0.03) {
            float tp = dot(u_bowA - ro, nrm) / den;
            float ts = 1e9;
            if (d < 1.0) {
                vec4 ws = u_invVP * vec4(ndc, d * 2.0 - 1.0, 1.0);
                ts = dot(ws.xyz / ws.w - ro, rd);
            }
            if (tp < ts) {
                vec3 rel = ro + rd * tp - u_bowA;
                vec2 pq = vec2(dot(rel, vec3(-nrm.z, 0.0, nrm.x)), rel.y);
                float band = (u_bowR - length(pq)) / (u_bowR * 0.2);
                if (band > 0.0 && band < 1.0 && pq.y > -0.5) {
                    // red on the outside to violet on the inside: the hue wheel from 0 to
                    // about three quarters round (any further runs on into magenta)
                    vec3 bow = clamp(abs(fract(band * 0.76 + vec3(0.0, 0.33, 0.67)) * 6.0 - 3.0) - 1.0, 0.0, 1.0);
                    // faint, and thinning out where it meets the ground
                    // (a plateau with soft edges, so the red and violet at the two edges
                    // are as strong as the colours between)
                    float k = smoothstep(0.0, 0.14, band) * smoothstep(0.0, 0.14, 1.0 - band) * smoothstep(-0.5, 2.5, pq.y);
                    c = mix(c, bow * 0.95 + c * 0.25, clamp(k * u_rainbow * 0.62 * abs(den) / (0.3 + abs(den)), 0.0, 0.7));
                }
            }
        }
    }
    // northern lights seen in the sky itself (where there's sky to see: past the far
    // edge of the world), fixed to compass directions; the ones over the land are
    // real curtains hung in the world (see `draw_aurora`), and cast their colour on it
    if (u_aurora > 0.01) {
        if (d >= 1.0) {
            vec3 au = clamp(auroraAt(skyDir(v_uv * 2.0 - 1.0, aspect)) * u_aurora, 0.0, 0.93);
            c = 1.0 - (1.0 - c) * (1.0 - au);
        }
        c *= mix(vec3(1.0), vec3(0.9, 1.1, 1.03), 0.3 * u_aurora);
    }
    // a shooting star
    if (u_meteorT > 0.0 && u_meteorT < 1.0) {
        vec2 head = u_meteor.xy + u_meteor.zw * u_meteorT;
        vec2 rel = (v_uv - head) * vec2(aspect, 1.0);
        vec2 dn = normalize(u_meteor.zw * vec2(aspect, 1.0));
        float along = dot(rel, -dn);
        float side = abs(rel.x * dn.y - rel.y * dn.x);
        if (along > 0.0 && along < 0.14) {
            c += vec3(1.0, 0.97, 0.9) * (1.0 - along / 0.14) * smoothstep(0.004, 0.0, side) * sin(u_meteorT * 3.14159);
        }
    }
    vec2 q = v_uv - 0.5;
    c *= 1.0 - dot(q, q) * 0.55;
    c += u_flash * vec3(0.75, 0.8, 0.95) * 0.55;
#if FX_GRADE
    // the look of the hour and the season, from a lookup table (which also
    // lifts the darks at night: dim screens and TVs crush them to black)
    c = texture(u_lut, clamp(c, 0.0, 1.0) * (15.0 / 16.0) + 0.5 / 16.0).rgb;
#else
    c = pow(max(c, vec3(0.0)), vec3(1.0 - 0.16 * u_night));
#endif
    c += (hash12(gl_FragCoord.xy + fract(u_time) * 100.0) - 0.5) / 255.0;
    o = vec4(c, 1.0);
}
"#;

/// Bright-pass and downsample for the bloom chain (dual Kawase).
pub const BLOOM_DOWN_FS: &str = r#"#version 300 es
precision highp float;
in vec2 v_uv;
uniform sampler2D u_src;
uniform vec2 u_texel;
uniform float u_first;
out vec4 o;
vec3 tap(vec2 uv) {
    vec3 c = texture(u_src, uv).rgb;
    // only what's brighter than white glows
    return u_first > 0.5 ? max(c - 0.9, 0.0) : c;
}
void main() {
    vec2 t = u_texel;
    vec3 s = tap(v_uv) * 4.0 + tap(v_uv + vec2(-t.x, -t.y)) + tap(v_uv + vec2(t.x, -t.y)) + tap(v_uv + vec2(-t.x, t.y)) + tap(v_uv + vec2(t.x, t.y));
    o = vec4(s / 8.0, 1.0);
}
"#;

/// Upsample with a tent filter, added onto the next level up.
pub const BLOOM_UP_FS: &str = r#"#version 300 es
precision highp float;
in vec2 v_uv;
uniform sampler2D u_src;
uniform vec2 u_texel;
uniform float u_w;
out vec4 o;
void main() {
    vec2 t = u_texel;
    vec3 s = texture(u_src, v_uv + vec2(-2.0 * t.x, 0.0)).rgb + texture(u_src, v_uv + vec2(2.0 * t.x, 0.0)).rgb
           + texture(u_src, v_uv + vec2(0.0, -2.0 * t.y)).rgb + texture(u_src, v_uv + vec2(0.0, 2.0 * t.y)).rgb
           + 2.0 * (texture(u_src, v_uv + vec2(-t.x, t.y)).rgb + texture(u_src, v_uv + vec2(t.x, t.y)).rgb
                  + texture(u_src, v_uv + vec2(-t.x, -t.y)).rgb + texture(u_src, v_uv + vec2(t.x, -t.y)).rgb);
    o = vec4(s / 12.0 * u_w, 1.0);
}
"#;

/// Temporal anti-aliasing: the new frame (drawn with a sub-pixel jiggle)
/// blended with last frame's result, reprojected through the depth and
/// clamped to the colours nearby so nothing ghosts.
pub const TAA_FS: &str = r#"#version 300 es
precision highp float;
in vec2 v_uv;
uniform sampler2D u_cur;
uniform sampler2D u_hist;
uniform sampler2D u_depth;
uniform vec2 u_texel;
uniform mat4 u_invVP;
uniform mat4 u_prevVP;
uniform float u_first;
out vec4 o;
void main() {
    vec3 cur = texture(u_cur, v_uv).rgb;
    if (u_first > 0.5) {
        o = vec4(cur, 1.0);
        return;
    }
    vec3 mn = cur;
    vec3 mx = cur;
    for (int i = -1; i <= 1; i++) {
        for (int j = -1; j <= 1; j++) {
            vec3 n = texture(u_cur, v_uv + vec2(float(i), float(j)) * u_texel).rgb;
            mn = min(mn, n);
            mx = max(mx, n);
        }
    }
    float d = texture(u_depth, v_uv).r;
    vec4 wp = u_invVP * vec4(v_uv * 2.0 - 1.0, d * 2.0 - 1.0, 1.0);
    vec4 pp = u_prevVP * vec4(wp.xyz / wp.w, 1.0);
    vec2 puv = pp.xy / pp.w * 0.5 + 0.5;
    if (puv.x < 0.0 || puv.x > 1.0 || puv.y < 0.0 || puv.y > 1.0) {
        o = vec4(cur, 1.0);
        return;
    }
    vec3 h = texture(u_hist, puv).rgb;
    vec3 hc = clamp(h, mn, mx);
    // the more the history had to be pulled in, the less it's trusted
    float trust = 1.0 - clamp(length(h - hc) * 2.0, 0.0, 0.7);
    o = vec4(mix(cur, hc, 0.94 * trust), 1.0);
}
"#;
