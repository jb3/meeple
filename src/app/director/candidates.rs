use super::super::*;

impl App {
    /// Everywhere within reach the autopilot might head for, each with how
    /// worth a visit it is by the time we'd get there: markets and weddings
    /// predicted from the towns' own dice, lighthouses at night, mills in a
    /// wind, harbours at dawn, places where railways meet.
    pub(in crate::app) fn candidates(&self, here: Vec2, hour: f32, day: i64, season: f32, sph: f32) -> Vec<Cand> {
        let gen = &self.world.gen;
        let r = 800;
        let (x0, z0, x1, z1) = (here.x as i32 - r, here.y as i32 - r, here.x as i32 + r, here.y as i32 + r);
        // game hour on arrival, walking the camera at a few units a second
        let eta = |p: Vec2| hour + p.distance(here) / 5.0 / sph;
        let w = &self.weather;
        let fine = !w.snowing && w.precip < 0.05 && w.storm < 0.1 && w.fog < 0.5;
        let mut out = Vec::new();
        for t in gen.towns_near(x0, z0, x1, z1, 0) {
            let p = Vec2::new(t.cx as f32, t.cz as f32);
            let e = eta(p);
            let mut weight = 0.5 + if t.city { 0.4 } else { 0.0 };
            let mut why = format!("{} of {}", if t.city { "The city" } else { "The village" }, t.name);
            let mut wait = None;
            let mut promise = None;
            if e < 24.0 {
                for f in forecast(&t, day, season) {
                    // no point heading for a wedding or christening that foul
                    // weather will keep from happening
                    if f.fine_only && !fine {
                        continue;
                    }
                    if e <= f.to - 0.4 {
                        let v = f.weight * if e >= f.from - 2.5 { 1.0 } else { 0.65 };
                        if v > weight {
                            weight = v;
                            why = f.why.clone();
                            wait = (e < f.from).then_some(f.from);
                            promise = f.occasion.map(|occasion| Promise { town: t.id, occasion, day, until: (f.to - 1.0).max(f.from), to: f.to });
                        }
                    }
                }
                if weight <= 1.0 && (17.0..21.0).contains(&e) {
                    weight = 0.95;
                    why = format!("Lamps coming on in {}", t.name);
                } else if weight <= 1.0 && (5.5..7.8).contains(&e) {
                    weight = 0.85;
                    why = format!("Dawn over {}", t.name);
                }
                if weight <= 1.3 && (2.95..4.0).contains(&season) && (10.0..19.0).contains(&e) && gen.green(&t).is_some() {
                    weight = 1.3;
                    why = format!("Skating on the green at {}", t.name);
                }
            }
            if w.snowing && t.style == Style::Northern {
                weight += 0.4;
            }
            if w.precip > 0.4 {
                weight -= 0.25;
            }
            out.push(Cand { pos: p, id: t.id, kind: if t.city { "city" } else { "village" }, weight, why, wait, promise });
        }
        // the next chapter of the story we're following, if it's time to set off
        if let Some(s) = &self.dir.story {
            let e = eta(s.pos);
            if let Some(b) = s.due(day, e) {
                let promise = b.occasion.map(|occasion| Promise { town: s.town, occasion, day, until: (b.to - 1.0).max(b.from), to: b.to });
                out.push(Cand { pos: s.pos, id: hash2(0x570a, s.town as i32, s.next as i32), kind: "story", weight: 3.8, why: format!("{} · {}: {}", s.chapter(), s.title, b.why), wait: (e < b.from).then_some(b.from), promise });
            }
        }
        for s in gen.sites_near(x0, z0, x1, z1) {
            let p = s.centre();
            let e = eta(p) % 24.0;
            let night = !(6.5..18.5).contains(&e);
            let dawn = (5.0..9.0).contains(&e);
            let dusk = (16.0..20.0).contains(&e);
            let windy = w.wind.length() > 0.35;
            let (weight, why, kind): (f32, String, &'static str) = match s.kind {
                SiteKind::Lighthouse if night => (2.2 + w.storm, "The lighthouse sweeps the sea".into(), "lighthouse"),
                SiteKind::Lighthouse if w.fog > 0.4 => (1.6, "A foghorn in the mist at the lighthouse".into(), "lighthouse"),
                SiteKind::Lighthouse => (0.6, "A lighthouse on the coast".into(), "lighthouse"),
                SiteKind::Harbour if dawn => (1.6, "Fishing boats putting out from the harbour".into(), "harbour"),
                SiteKind::Harbour if dusk => (1.5, "Boats coming home to the harbour".into(), "harbour"),
                SiteKind::Harbour => (0.6, "A quiet harbour".into(), "harbour"),
                SiteKind::Windmill if windy => (1.4, "Windmill sails turning in the wind".into(), "mill"),
                SiteKind::Windmill => (0.5, "A windmill on the hill".into(), "mill"),
                SiteKind::Watermill => (0.7 + w.precip * 0.8, "The water wheel turning".into(), "mill"),
                SiteKind::Ruin if (night || w.fog > 0.3) && crate::sim::wonders::haunted(&s) => (1.8, "Cold lights wander the haunted ruins".into(), "ruin"),
                SiteKind::Ruin if night || w.fog > 0.3 => (1.3, "Old ruins in the gloom".into(), "ruin"),
                SiteKind::Ruin => (0.45, "An old ruin".into(), "ruin"),
                SiteKind::Quarry => (0.5, "The quarry".into(), "quarry"),
                SiteKind::Manor => (0.55, "A manor house".into(), "manor"),
                SiteKind::StoneCircle if dawn => (1.6, "Dawn at the standing stones".into(), "ancient"),
                SiteKind::StoneCircle => (0.8, "The standing stones".into(), "ancient"),
                SiteKind::ChalkFigure => (0.95, "A giant cut into the chalk hillside".into(), "ancient"),
                SiteKind::Geyser => (1.5, "Hot springs and a geyser".into(), "wonder"),
                SiteKind::Oasis => (1.3, "A camel caravan at the oasis".into(), "wonder"),
                SiteKind::Mine => (1.1, "Winding gear turning at the hill mine".into(), "wonder"),
                SiteKind::SkiResort if (2.9..4.0).contains(&season) => (1.7, "Skiers at the mountain resort".into(), "wonder"),
                SiteKind::SkiResort => (0.8, "A mountain chalet resort".into(), "wonder"),
                SiteKind::Monastery if [6.0, 12.0, 18.0].iter().any(|b| (e - b).abs() < 0.6) => (1.6, "The monastery bell rings the hour".into(), "holy"),
                SiteKind::Monastery => (0.75, "A monastery on the hill".into(), "holy"),
                SiteKind::Shrine if night => (1.0, "A shrine lamp burning in the dark".into(), "holy"),
                SiteKind::Shrine => (0.4, "A wayside shrine".into(), "holy"),
                SiteKind::HillFort if dusk => (1.2, "Lamps lit at the hill fort".into(), "fort"),
                SiteKind::HillFort => (0.8, "A hill fort on the heights".into(), "fort"),
                SiteKind::Camp if dusk || night => (1.3, "Travellers round their campfire".into(), "camp"),
                SiteKind::Camp => (0.5, "A travellers' camp".into(), "camp"),
                SiteKind::Orchard if (2.0..2.95).contains(&season) && (8.0..17.0).contains(&e) => (1.5, "Apple picking in the orchard".into(), "farm"),
                SiteKind::Orchard => (0.5, "An orchard".into(), "farm"),
                SiteKind::Apiary if fine && (7.0..18.0).contains(&e) && !(2.95..4.0).contains(&season) => (1.1, "Bees busy among the hives".into(), "farm"),
                SiteKind::Apiary => (0.4, "An apiary".into(), "farm"),
            };
            out.push(Cand { pos: p, id: hash2(0x517e, s.x, s.z), kind, weight, why, wait: None, promise: None });
        }
        // weirs, busy with leaping salmon in autumn
        for w in gen.weirs_near(x0, z0, x1, z1) {
            let (weight, why) = if (2.0..3.0).contains(&(season % 4.0)) {
                (1.8, "Salmon leaping the weir")
            } else {
                (0.5, "Water tumbling over a weir")
            };
            out.push(Cand { pos: w.c, id: hash2(0x3e17, w.c.x as i32, w.c.y as i32), kind: "weir", weight, why: why.into(), wait: None, promise: None });
        }
        // a few places where the railways meet or pass one another
        let mut rails: Vec<(f32, Cand)> = Vec::new();
        for (_, _, c) in gen.crossings_near(x0, z0, x1, z1) {
            let d = c.distance(here);
            let jn = gen.chords_near(c.x as i32 - 2, c.y as i32 - 2, c.x as i32 + 2, c.y as i32 + 2).iter().any(|k| k.a != k.b);
            rails.push((d, Cand { pos: c, id: hash2(0x7a11, c.x as i32, c.y as i32), kind: "rail", weight: if jn { 1.1 } else { 0.8 }, why: if jn { "Where two railways join".into() } else { "Railways crossing".into() }, wait: None, promise: None }));
        }
        for c in gen.chords_near(x0, z0, x1, z1) {
            if c.a == c.b {
                let m = c.at(c.length() * 0.5);
                rails.push((m.distance(here), Cand { pos: m, id: c.id, kind: "rail", weight: 1.0, why: "A passing loop on the line".into(), wait: None, promise: None }));
            }
        }
        rails.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        out.extend(rails.into_iter().take(6).map(|r| r.1));
        // wild country of the rarer sorts: dunes, frozen moors, jungle, high mountains
        for k in 0..16 {
            let a = k as f32 * std::f32::consts::TAU / 16.0;
            let p = here + Vec2::new(a.cos(), a.sin()) * (330.0 + 40.0 * (k % 3) as f32 * 3.0);
            let h = gen.raw_height(p.x, p.y);
            let why = match gen.biome(p.x, p.y, h) {
                Biome::Desert if h > SEA as f32 => "Out across the dunes",
                Biome::Tundra if h > SEA as f32 => "Crossing the frozen moors",
                Biome::Mountain => "Up in the high mountains",
                Biome::Jungle if h > SEA as f32 => "Deep in the jungle",
                _ => continue,
            };
            out.push(Cand { pos: p, id: hash2(0xb10e, (p.x / 160.0).floor() as i32, (p.y / 160.0).floor() as i32), kind: "biome", weight: 1.3, why: why.into(), wait: None, promise: None });
        }
        // a wander along a canal
        if let Some(c) = gen.canals_near(here.y as i32 - 500, here.y as i32 + 500).into_iter().find_map(|cz| {
            (-25..=25).map(|k| here.x + k as f32 * 20.0).filter(|x| (x - here.x).abs() > 120.0 || (cz as f32 - here.y).abs() > 120.0).find(|x| {
                (0..5).all(|j| gen.canal_open(x + j as f32 * 10.0 - 20.0, cz))
            }).map(|x| Vec2::new(x, cz as f32 + 0.5))
        }) {
            let e = eta(c) % 24.0;
            let weight = if (6.0..19.0).contains(&e) { 1.0 } else { 0.6 };
            out.push(Cand { pos: c, id: hash2(0xca4a, c.x as i32 / 40, c.y as i32 / 40), kind: "canal", weight, why: "A stroll along the canal".into(), wait: None, promise: None });
        }
        out
    }

    /// Somewhere far off worth the long way round: a town or a rarer place
    /// well beyond the usual reach.
    pub(in crate::app) fn far_candidate(&mut self, here: Vec2) -> Option<Cand> {
        let gen = &self.world.gen;
        let r = 1700;
        let (x0, z0, x1, z1) = (here.x as i32 - r, here.y as i32 - r, here.x as i32 + r, here.y as i32 + r);
        let mut out: Vec<Cand> = Vec::new();
        for t in gen.towns_near(x0, z0, x1, z1, 0) {
            let p = Vec2::new(t.cx as f32, t.cz as f32);
            out.push(Cand { pos: p, id: t.id, kind: if t.city { "city" } else { "village" }, weight: 1.0, why: format!("{} of {}", if t.city { "The city" } else { "The village" }, t.name), wait: None, promise: None });
        }
        for s in gen.sites_near(x0, z0, x1, z1) {
            let (why, kind) = match s.kind {
                SiteKind::Lighthouse => ("A far lighthouse on the coast", "lighthouse"),
                SiteKind::Geyser => ("Hot springs far away", "wonder"),
                SiteKind::Oasis => ("An oasis across the desert", "wonder"),
                SiteKind::SkiResort => ("A mountain resort", "wonder"),
                SiteKind::Mine => ("A mine in the far hills", "wonder"),
                SiteKind::StoneCircle => ("Distant standing stones", "ancient"),
                SiteKind::Harbour => ("A harbour on the far coast", "harbour"),
                _ => continue,
            };
            out.push(Cand { pos: s.centre(), id: hash2(0x517e, s.x, s.z), kind, weight: 1.2, why: why.into(), wait: None, promise: None });
        }
        let d = &self.dir;
        out.retain(|c| {
            let dist = c.pos.distance(here);
            (650.0..1700.0).contains(&dist) && !d.visited.contains(&c.id)
        });
        if out.is_empty() {
            return None;
        }
        let i = self.dir.rng.next_u32() as usize % out.len();
        Some(out.swap_remove(i))
    }
}
