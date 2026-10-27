//! Building lots: how a town's streets are filled with houses, shops and the like.

use super::super::*;

impl Gen {
    pub fn lots(&self, t: &Town) -> Rc<Vec<Lot>> {
        if let Some(l) = self.lots.borrow().get(&t.id) {
            return l.clone();
        }
        let l = Rc::new(self.make_lots(t));
        self.lots.borrow_mut().insert(t.id, l.clone());
        l
    }

    pub(in crate::world::gen) fn make_lots(&self, t: &Town) -> Vec<Lot> {
        let s = STREET;
        let n = t.radius / s + 1;
        let r = t.radius as f32;
        let roads = self.roads_near(t.cx - t.radius, t.cz - t.radius, t.cx + t.radius, t.cz + t.radius);
        let rails = self.rails_near(t.cx - t.radius, t.cz - t.radius, t.cx + t.radius, t.cz + t.radius);
        let chords = self.chords_near(t.cx - t.radius, t.cz - t.radius, t.cx + t.radius, t.cz + t.radius);
        let mut cands = Vec::new();
        for i in -n..n {
            for j in -n..n {
                if (i == -1 || i == 0) && (j == -1 || j == 0) {
                    continue;
                }
                let x0 = t.cx + i * s + 2;
                let z0 = t.cz + j * s + 2;
                let (x1, z1) = (x0 + 7, z0 + 7);
                let mx = x0 as f32 + 4.0 - t.cx as f32;
                let mz = z0 as f32 + 4.0 - t.cz as f32;
                let d = (mx * mx + mz * mz).sqrt();
                if d > r - 1.0 {
                    continue;
                }
                let lot_mid = Vec2::new(x0 as f32 + 4.0, z0 as f32 + 4.0);
                // a lot is 8x8: anything within 6.5 of its edge is within ~12 of its middle
                if rails.iter().any(|l| lot_near_rail(l, x0, z0, x1, z1))
                    || chords.iter().any(|c| c.dist(lot_mid) < 12.5)
                {
                    continue;
                }
                if roads.iter().any(|sg| sg.hits_rect(x0, z0, x1, z1, 2) && !self.own_road(t, sg)) {
                    continue;
                }
                cands.push((d, i, j, x0, z0, mx, mz));
            }
        }
        cands.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

        let mut lots = Vec::new();
        for (idx, (d, i, j, x0, z0, mx, mz)) in cands.into_iter().enumerate() {
            let seed = hash3(t.id, i, 9, j);
            let hr = unit(seed);
            let frac = d / r;
            let kind = match idx {
                0 => LotKind::Tavern,
                1 => LotKind::Bakery,
                2 => LotKind::Smithy,
                3 if t.city => LotKind::Church,
                3 => LotKind::Shop,
                4 if t.city => LotKind::Shop,
                4 => LotKind::School,
                5 if t.city => LotKind::School,
                6 if t.city => LotKind::Library,
                _ if !t.city && frac > 0.55 && hr < 0.6 => LotKind::Farm,
                _ if t.city && frac < 0.62 && hr < 0.7 => LotKind::Apartment,
                _ if hr > 0.93 => LotKind::Garden,
                _ => LotKind::House,
            };
            let face = if mx.abs() > mz.abs() {
                if mx > 0.0 { 3 } else { 1 }
            } else if mz > 0.0 {
                0
            } else {
                2
            };
            let h2 = hash_u32(seed ^ 0x55);
            let pick = |opts: &[u8], k: u32| opts[k as usize % opts.len()];
            let wall = match (kind, t.style) {
                (LotKind::Apartment, _) => pick(&[BRICK, PLASTER, STONEBRICK, PLASTER_Y], h2),
                (LotKind::Church | LotKind::Library, _) => STONEBRICK,
                (LotKind::Tavern, Style::Northern) => LOG,
                (LotKind::Tavern, _) => PLANK,
                (LotKind::Bakery, _) => BRICK,
                (LotKind::Smithy, _) => WALLSTONE,
                (LotKind::School, _) => pick(&[BRICK, PLASTER_Y], h2),
                (_, Style::Highland) => pick(&[WALLSTONE, STONEBRICK, WALLSTONE, COBBLE], h2),
                (_, Style::Coastal) => pick(&[PLASTER, PLASTER, PLASTER, PLASTER_Y], h2),
                (_, Style::Northern) => pick(&[LOG, LOG, PLANK], h2),
                (_, Style::Mediterranean) => pick(&[PLASTER, PLASTER, PLASTER_Y, PLASTER, PLASTER_P], h2),
                (_, Style::Tropical) => pick(&[PLANK, PLANK, LOG, PLASTER_Y], h2),
                _ if t.city => pick(&[BRICK, PLASTER, PLASTER_Y, PLASTER_P, STONEBRICK], h2),
                _ => pick(&[PLASTER, PLANK, PLASTER_Y, PLASTER, BRICK, PLASTER_P], h2),
            };
            let roof = match t.style {
                Style::Highland => pick(&[ROOF_SLATE, ROOF_SLATE, ROOF_BROWN], h2 >> 8),
                Style::Coastal => pick(&[ROOF_BLUE, ROOF_BLUE, ROOF_SLATE, ROOF_RED], h2 >> 8),
                Style::Northern => pick(&[ROOF_BROWN, ROOF_GREEN, ROOF_SLATE], h2 >> 8),
                Style::Mediterranean => pick(&[ROOF_RED, ROOF_RED, ROOF_RED, ROOF_BROWN], h2 >> 8),
                Style::Tropical => pick(&[ROOF_GREEN, ROOF_GREEN, ROOF_BROWN], h2 >> 8),
                Style::Lowland => pick(&[ROOF_RED, ROOF_BLUE, ROOF_BROWN, ROOF_GREEN, ROOF_SLATE, ROOF_RED], h2 >> 8),
            };
            let floors = match kind {
                LotKind::Apartment => 3 + (h2 >> 12) as i32 % 3,
                LotKind::Tavern | LotKind::Library | LotKind::Manor => 2,
                LotKind::House if t.city => 1 + (h2 >> 12) as i32 % 2,
                LotKind::House => 1 + ((h2 >> 12) % 4 == 0) as i32,
                _ => 1,
            };
            let mut lot = Lot {
                kind,
                x0,
                z0,
                x1: x0 + 7,
                z1: z0 + 7,
                face,
                door: (0, 0),
                front: (0.0, 0.0),
                street: (0.0, 0.0),
                floors,
                wall,
                roof,
                seed,
            };
            let (fx0, fz0, fx1, fz1) = lot.footprint();
            let midx = (fx0 + fx1 + 1) / 2;
            let midz = (fz0 + fz1 + 1) / 2;
            let (door, front, street) = match face {
                0 => ((midx, fz0), (midx as f32 + 0.5, fz0 as f32 - 0.5), (midx as f32 + 0.5, z0 as f32 - 1.5)),
                2 => ((midx, fz1), (midx as f32 + 0.5, fz1 as f32 + 1.5), (midx as f32 + 0.5, lot.z1 as f32 + 2.5)),
                1 => ((fx1, midz), (fx1 as f32 + 1.5, midz as f32 + 0.5), (lot.x1 as f32 + 2.5, midz as f32 + 0.5)),
                _ => ((fx0, midz), (fx0 as f32 - 0.5, midz as f32 + 0.5), (x0 as f32 - 1.5, midz as f32 + 0.5)),
            };
            lot.door = door;
            lot.front = front;
            lot.street = street;
            lots.push(lot);
        }
        lots
    }

    /// Roads that start at this town run along its own street lines.
    pub(in crate::world::gen) fn own_road(&self, t: &Town, s: &Seg) -> bool {
        (s.x0 == s.x1 && s.x0 == t.cx) || (s.z0 == s.z1 && s.z0 == t.cz)
    }
}
