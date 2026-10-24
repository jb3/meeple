use super::*;

/// Blocks of a tree not joined face-to-face to its trunk.
fn strays(f: fn(&mut Chunk, i32, i32, i32, u32), k: u32) -> usize {
    let mut ch = Chunk::new(0, 0);
    f(&mut ch, 8, 10, 8, k);
    let cells: Vec<(i32, i32, i32)> = (0..CH)
        .flat_map(|y| (-1..=CS).flat_map(move |z| (-1..=CS).map(move |x| (x, y, z))))
        .filter(|&(x, y, z)| ch.get(x, y, z) != AIR)
        .collect();
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![(8, 10, 8)];
    while let Some(c) = stack.pop() {
        if !cells.contains(&c) || !seen.insert(c) {
            continue;
        }
        for (dx, dy, dz) in [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)] {
            stack.push((c.0 + dx, c.1 + dy, c.2 + dz));
        }
    }
    cells.len() - seen.len()
}

#[test]
fn trees_are_one_piece() {
    for (name, f) in [("oak", oak as fn(&mut Chunk, i32, i32, i32, u32)), ("birch", birch), ("pine", pine)] {
        let bad: Vec<(u32, usize)> = (0..300u32).map(|k| hash_u32(k)).map(|k| (k, strays(f, k))).filter(|b| b.1 > 0).collect();
        assert!(bad.is_empty(), "{name}: {} of 300 have loose blocks, e.g. {:?}", bad.len(), &bad[..bad.len().min(4)]);
    }
}
