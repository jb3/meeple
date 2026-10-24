use super::*;

#[test]
fn carts_appear_just_out_of_sight() {
    let pts = [Vec2::new(-200.0, 0.0), Vec2::new(0.0, 0.0), Vec2::new(0.0, 200.0)];
    let (seg, p) = entry_point(&pts, Vec2::new(0.0, 50.0), 60.0).unwrap();
    assert_eq!(seg, 0);
    let d = p.distance(Vec2::new(0.0, 50.0));
    assert!((60.0..64.0).contains(&d), "entry {p} is {d} from view");
}

#[test]
fn no_entry_if_the_road_starts_on_screen_or_never_arrives() {
    let pts = [Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0)];
    assert!(entry_point(&pts, Vec2::new(10.0, 0.0), 60.0).is_none());
    assert!(entry_point(&pts, Vec2::new(0.0, 500.0), 60.0).is_none());
}
