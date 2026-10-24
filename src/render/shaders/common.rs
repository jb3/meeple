pub const COMMON: &str = r#"
precision highp float;
precision highp sampler2DShadow;
uniform highp sampler2DShadow u_shadow;
uniform mat4 u_lightVP;
uniform vec2 u_shadowTexel;
uniform float u_shadowTaps;
#if FX_CASCADE
// a second, tighter map round the camera, for crisper shadows close up
uniform highp sampler2DShadow u_shadow2;
uniform mat4 u_lightVP2;
uniform vec2 u_shadowTexel2;
#endif
uniform vec3 u_sunDir;
uniform vec3 u_sunCol;
uniform vec3 u_ambSky;
uniform vec3 u_ambGround;
uniform float u_time;
uniform float u_cloud;
uniform float u_night;
uniform float u_lamp;
uniform vec2 u_wind;
uniform vec4 u_lights[16];
uniform int u_nlights;

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
float hash12(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
float vnoise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash12(i);
    float b = hash12(i + vec2(1.0, 0.0));
    float c = hash12(i + vec2(0.0, 1.0));
    float d = hash12(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
BAYER
float shadowTaps(highp sampler2DShadow tex, vec2 texel, vec3 p) {
    float z = p.z - 0.0006;
    if (u_shadowTaps < 2.0) return texture(tex, vec3(p.xy, z));
    vec2 t = texel * 0.75;
    float s = texture(tex, vec3(p.xy + vec2(-t.x, -t.y), z));
    s += texture(tex, vec3(p.xy + vec2(t.x, -t.y), z));
    s += texture(tex, vec3(p.xy + vec2(-t.x, t.y), z));
    s += texture(tex, vec3(p.xy + vec2(t.x, t.y), z));
    return s * 0.25;
}
float shadowAt(vec3 wp, vec3 n) {
    if (u_shadowTaps < 0.5) return 1.0;
    vec3 wo = wp + n * 0.05;
    vec4 lp = u_lightVP * vec4(wo, 1.0);
    vec3 p = lp.xyz / lp.w * 0.5 + 0.5;
    if (p.x <= 0.0 || p.x >= 1.0 || p.y <= 0.0 || p.y >= 1.0 || p.z >= 1.0) return 1.0;
    // fade out towards the edge of the map, so where the shadows stop there's
    // no visible seam
    float edge = min(min(p.x, 1.0 - p.x), min(p.y, 1.0 - p.y));
    float fade = smoothstep(0.0, 0.12, edge);
#if FX_CASCADE
    vec4 lp2 = u_lightVP2 * vec4(wo, 1.0);
    vec3 q = lp2.xyz / lp2.w * 0.5 + 0.5;
    float e2 = min(min(q.x, 1.0 - q.x), min(q.y, 1.0 - q.y));
    if (e2 > 0.0 && q.z < 1.0) {
        float w2 = smoothstep(0.0, 0.1, e2);
        float near = shadowTaps(u_shadow2, u_shadowTexel2, q);
        if (w2 >= 0.999) return near;
        return mix(mix(1.0, shadowTaps(u_shadow, u_shadowTexel, p), fade), near, w2);
    }
#endif
    return mix(1.0, shadowTaps(u_shadow, u_shadowTexel, p), fade);
}
#if FX_CLOUDS
// A deck of cloud 55 up: its shadows land displaced away from the sun, more
// for the ground than for a rooftop, with sharp edges on broken cloud and soft
// broad sheets when it's overcast.
float cloudShade(vec3 wp) {
    vec2 par = u_sunDir.xz * (55.0 - wp.y) / max(u_sunDir.y, 0.25);
    vec2 q = (wp.xz + par) / 90.0 + u_wind * u_time * 0.012;
    float n = vnoise(q) * 0.55 + vnoise(q * 2.7 + 5.0) * 0.3 + vnoise(q * 6.1 + 9.0) * 0.15;
    float soft = mix(0.1, 0.4, smoothstep(0.5, 1.0, u_cloud));
    float cov = smoothstep(1.0 - u_cloud, 1.0 - u_cloud + soft, n);
    return 1.0 - cov * 0.6 * smoothstep(0.05, 0.4, u_cloud);
}
#else
float cloudShade(vec3 wp) {
    vec2 q = wp.xz / 90.0 + u_wind * u_time * 0.012;
    float n = vnoise(q) * 0.65 + vnoise(q * 2.7 + 5.0) * 0.35;
    float cov = smoothstep(1.0 - u_cloud, 1.0 - u_cloud + 0.3, n);
    return 1.0 - cov * 0.6 * smoothstep(0.05, 0.4, u_cloud);
}
#endif
vec3 lamps(vec3 wp, vec3 n) {
    vec3 c = vec3(0.0);
    if (u_lamp < 0.01) return c;
    for (int i = 0; i < 16; i++) {
        if (i >= u_nlights) break;
        vec3 d = u_lights[i].xyz - wp;
        float d2 = dot(d, d);
        float att = u_lights[i].w / (1.0 + d2 * 0.22);
        float nd = max(dot(n, d * inversesqrt(max(d2, 1e-4))), 0.0) * 0.6 + 0.4;
        c += vec3(1.0, 0.7, 0.4) * att * nd;
    }
    return c * u_lamp;
}
vec3 lighting(vec3 wp, vec3 n, float ao) {
    float ndl = max(dot(n, u_sunDir), 0.0);
    float sh = ndl > 0.0 ? shadowAt(wp, n) : 0.0;
    vec3 amb = mix(u_ambGround, u_ambSky, n.y * 0.5 + 0.5) * (0.45 + 0.55 * ao);
    return amb + u_sunCol * ndl * sh * cloudShade(wp) * (0.8 + 0.2 * ao) + lamps(wp, n);
}
"#;

/// Foliage sway, shared by the colour and shadow passes so leaves never
/// shadow themselves as they move. Substituted for `SWAY` in vertex shaders.
pub const SWAY: &str = r#"
vec3 sway(vec3 p, float m) {
    if (m == 2.0 || m == 8.0) {
        float w = 0.025 + length(u_wind) * 0.05;
        p.x += sin(u_time * 1.6 + p.z * 0.45 + p.y * 0.3) * w;
        p.z += cos(u_time * 1.3 + p.x * 0.4) * w * 0.6;
    }
    if (m == 12.0 || m == 6.0 || m == 7.0) {
        // grass and crops ripple in gusts that roll across the land
        float up = m == 7.0 ? 0.9 : fract(p.y + 0.001);
        float wl = length(u_wind);
        vec2 wd = wl > 0.01 ? u_wind / wl : vec2(1.0, 0.0);
        float gust = 0.55 + 0.45 * sin(dot(p.xz, wd) * 0.18 - u_time * 1.7);
        float amp = (0.05 + wl * 0.24) * gust * up;
        p.xz += wd * amp * (0.6 + 0.4 * sin(u_time * 3.1 + p.x * 1.3 + p.z));
    }
    return p;
}
"#;

/// Ordered 4x4 dither threshold for screen-door fading. Substituted for
/// `BAYER` where a shader needs it.
pub const BAYER: &str = r#"
float bayer4(vec2 p) {
    ivec2 q = ivec2(mod(p, 4.0));
    const float m[16] = float[16](0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0,
                                  3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
    return (m[q.x + q.y * 4] + 0.5) / 16.0;
}
"#;
