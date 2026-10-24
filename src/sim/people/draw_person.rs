use super::*;

pub(super) fn draw_person(b: &mut Boxes, fx: &mut Boxes, p: &Person, umbrella: bool, cam_yaw: f32, time: f32) {
    // wedding clothes: a white dress and veil, a dark suit
    let mut look = p.look;
    match p.wed {
        Wed::Bride => {
            look.shirt = [0.97, 0.96, 0.93];
            look.pants = [0.97, 0.96, 0.93];
            look.hat = Hat::Bun;
        }
        Wed::Groom => {
            look.shirt = [0.16, 0.17, 0.24];
            look.pants = [0.13, 0.13, 0.17];
            look.hat = Hat::None;
        }
        _ => {}
    }
    let l = &look;
    let walking = p.state == State::Walking;
    let sitting = p.state == State::Sitting;
    let working = p.state == State::Working;
    let dancing = p.state == State::Dancing;
    let singing = p.state == State::Singing;
    let playing = p.state == State::Playing;
    let sw = if walking || dancing { p.phase.sin() } else { 0.0 };
    let bob = if walking {
        (p.phase * 2.0).sin().abs() * 0.035
    } else if dancing {
        (p.phase * 2.0).sin().abs() * 0.13
    } else if singing {
        (p.phase * 0.7).sin() * 0.015
    } else {
        0.0
    };
    let drop = if sitting { 0.36 } else { 0.0 };
    let mut r = Rig::new(b, Vec3::new(p.pos.x, p.y, p.pos.y), p.yaw, l.scale);
    r.alpha = p.fade;
    let shirt_dark = [l.shirt[0] * 0.85, l.shirt[1] * 0.85, l.shirt[2] * 0.85];
    let out_and_about = matches!(p.goal, Goal::Plaza | Goal::Errand | Goal::Visit | Goal::Shop);
    let carry = if p.carry == Carry::Nothing && p.pram() && out_and_about && !sitting { Carry::Pram } else { p.carry };

    if sitting {
        for x in [-0.11, 0.11] {
            r.part(Vec3::new(x, 0.44, 0.16), Vec3::new(0.09, 0.08, 0.2), l.pants);
            r.part(Vec3::new(x, 0.22, 0.34), Vec3::new(0.08, 0.2, 0.08), l.pants);
        }
    } else {
        r.part(Vec3::new(0.11, 0.28, sw * 0.13), Vec3::new(0.09, 0.28, 0.09), l.pants);
        r.part(Vec3::new(-0.11, 0.28, -sw * 0.13), Vec3::new(0.09, 0.28, 0.09), l.pants);
    }
    let ty = 0.82 + bob - drop;
    r.part(Vec3::new(0.0, ty, 0.0), Vec3::new(0.22, 0.26, 0.14), l.shirt);

    // arms and tools
    let wt = time * 5.0 + p.phase;
    if working && p.role == Role::Smith {
        let s = (time * std::f32::consts::TAU / 1.1).sin();
        r.part(Vec3::new(0.3, ty + 0.15 + s * 0.15, 0.18), Vec3::new(0.07, 0.07, 0.22), shirt_dark);
        r.part(Vec3::new(0.3, ty + 0.25 + s * 0.3, 0.45), Vec3::new(0.08, 0.08, 0.12), [0.25, 0.25, 0.3]);
        r.part(Vec3::new(-0.3, ty, 0.0), Vec3::new(0.07, 0.24, 0.07), shirt_dark);
    } else if working && p.role == Role::Farmer {
        let s = wt.sin() * 0.5 + 0.5;
        for x in [-0.25, 0.25] {
            r.part(Vec3::new(x, ty + 0.02, 0.2), Vec3::new(0.07, 0.07, 0.2), shirt_dark);
        }
        r.part(Vec3::new(0.0, ty - 0.1 + s * 0.35, 0.42), Vec3::new(0.03, 0.4, 0.03), [0.55, 0.4, 0.25]);
        r.part(Vec3::new(0.0, ty - 0.48 + s * 0.35, 0.5), Vec3::new(0.12, 0.03, 0.06), [0.5, 0.5, 0.55]);
    } else if dancing {
        // arms raised, swinging in time with the steps
        let s = (p.phase * 2.0).sin();
        r.part(Vec3::new(0.32, ty + 0.36 + s * 0.06, 0.0), Vec3::new(0.07, 0.24, 0.07), shirt_dark);
        r.part(Vec3::new(-0.32, ty + 0.36 - s * 0.06, 0.0), Vec3::new(0.07, 0.24, 0.07), shirt_dark);
    } else if playing {
        // a lute: body against the chest, neck to the left, strumming hand
        let s = (p.phase * 3.0).sin();
        let wood = [0.62, 0.42, 0.24];
        r.part(Vec3::new(0.05, ty - 0.08, 0.22), Vec3::new(0.17, 0.2, 0.06), wood);
        r.part(Vec3::new(0.06, ty - 0.08, 0.285), Vec3::new(0.05, 0.05, 0.01), [0.2, 0.14, 0.1]);
        r.part(Vec3::new(-0.28, ty + 0.14, 0.22), Vec3::new(0.18, 0.035, 0.035), wood);
        r.part(Vec3::new(0.24, ty - 0.1 + s * 0.06, 0.3), Vec3::new(0.07, 0.07, 0.07), l.skin);
        r.part(Vec3::new(0.3, ty - 0.02, 0.08), Vec3::new(0.07, 0.2, 0.07), shirt_dark);
        r.part(Vec3::new(-0.3, ty + 0.06, 0.16), Vec3::new(0.07, 0.07, 0.18), shirt_dark);
    } else if singing {
        // hands clasped in front
        r.part(Vec3::new(0.2, ty - 0.04, 0.16), Vec3::new(0.07, 0.07, 0.16), shirt_dark);
        r.part(Vec3::new(-0.2, ty - 0.04, 0.16), Vec3::new(0.07, 0.07, 0.16), shirt_dark);
    } else if carry == Carry::Pram {
        // both hands on the pram's handle
        for x in [-0.15, 0.15] {
            r.part(Vec3::new(x, ty - 0.02, 0.17), Vec3::new(0.07, 0.07, 0.17), shirt_dark);
        }
    } else if umbrella && !sitting {
        r.part(Vec3::new(0.3, ty - 0.02, -sw * 0.1), Vec3::new(0.07, 0.24, 0.07), shirt_dark);
        r.part(Vec3::new(-0.24, ty + 0.12, 0.14), Vec3::new(0.07, 0.07, 0.17), shirt_dark);
    } else {
        r.part(Vec3::new(0.3, ty - 0.02, -sw * 0.14), Vec3::new(0.07, 0.24, 0.07), shirt_dark);
        r.part(Vec3::new(-0.3, ty - 0.02, sw * 0.14), Vec3::new(0.07, 0.24, 0.07), shirt_dark);
    }

    match carry {
        Carry::Pram => {
            // a coach-built pram, pushed in front
            let body = [0.2, 0.25, 0.4];
            r.part(Vec3::new(0.0, 0.5, 0.62), Vec3::new(0.2, 0.15, 0.3), body);
            r.part(Vec3::new(0.0, 0.72, 0.42), Vec3::new(0.2, 0.12, 0.1), body);
            r.part(Vec3::new(0.0, 0.67, 0.72), Vec3::new(0.15, 0.03, 0.15), [0.97, 0.95, 0.9]);
            for (x, z) in [(-0.18, 0.4), (0.18, 0.4), (-0.18, 0.84), (0.18, 0.84)] {
                r.part(Vec3::new(x, 0.12, z), Vec3::new(0.025, 0.11, 0.11), [0.12, 0.12, 0.14]);
            }
            r.part(Vec3::new(0.0, ty - 0.05, 0.3), Vec3::new(0.18, 0.025, 0.025), [0.12, 0.12, 0.14]);
        }
        Carry::Basket | Carry::Bread => {
            let fill = if p.carry == Carry::Bread { [0.8, 0.58, 0.32] } else { [0.4, 0.7, 0.3] };
            r.part(Vec3::new(0.34, ty - 0.32, 0.1), Vec3::new(0.13, 0.08, 0.1), [0.62, 0.48, 0.28]);
            r.part(Vec3::new(0.34, ty - 0.22, 0.1), Vec3::new(0.1, 0.04, 0.08), fill);
        }
        Carry::Nothing => {}
    }
    if p.role == Role::Postie && p.goal == Goal::Work {
        // a leather satchel on the hip
        r.part(Vec3::new(-0.26, ty - 0.15, 0.0), Vec3::new(0.06, 0.14, 0.16), [0.45, 0.3, 0.18]);
    }

    // head
    let hy = 1.27 + bob - drop;
    r.part(Vec3::new(0.0, hy, 0.0), Vec3::new(0.17, 0.17, 0.17), l.skin);
    for x in [-0.07, 0.07] {
        r.part(Vec3::new(x, hy + 0.02, 0.17), Vec3::new(0.025, 0.035, 0.01), [0.1, 0.08, 0.08]);
    }
    r.part(Vec3::new(0.0, hy + 0.16, -0.02), Vec3::new(0.18, 0.05, 0.18), l.hair);
    r.part(Vec3::new(0.0, hy + 0.02, -0.16), Vec3::new(0.18, 0.14, 0.04), l.hair);
    match l.hat {
        Hat::Straw => {
            r.part(Vec3::new(0.0, hy + 0.19, 0.0), Vec3::new(0.32, 0.025, 0.32), l.hat_col);
            r.part(Vec3::new(0.0, hy + 0.25, 0.0), Vec3::new(0.17, 0.06, 0.17), l.hat_col);
        }
        Hat::Chef => r.part(Vec3::new(0.0, hy + 0.3, 0.0), Vec3::new(0.16, 0.14, 0.16), l.hat_col),
        Hat::Cap => {
            r.part(Vec3::new(0.0, hy + 0.2, 0.0), Vec3::new(0.19, 0.05, 0.19), l.hat_col);
            r.part(Vec3::new(0.0, hy + 0.16, 0.2), Vec3::new(0.15, 0.02, 0.08), l.hat_col);
        }
        Hat::Bun => r.part(Vec3::new(0.0, hy + 0.24, -0.1), Vec3::new(0.08, 0.08, 0.08), l.hair),
        Hat::None => {}
    }
    match p.wed {
        Wed::Bride => {
            r.part(Vec3::new(0.0, hy - 0.05, -0.22), Vec3::new(0.2, 0.28, 0.025), [1.0, 1.0, 0.98]);
            r.part(Vec3::new(0.0, ty - 0.3, -0.02), Vec3::new(0.26, 0.2, 0.18), l.shirt);
            // a posy of flowers
            r.part(Vec3::new(0.0, ty - 0.06, 0.22), Vec3::new(0.08, 0.08, 0.06), [0.95, 0.55, 0.65]);
        }
        Wed::Groom => {
            r.part(Vec3::new(0.0, ty + 0.12, 0.145), Vec3::new(0.07, 0.12, 0.01), [0.97, 0.97, 0.95]);
            r.part(Vec3::new(0.12, ty + 0.16, 0.15), Vec3::new(0.03, 0.03, 0.01), [0.95, 0.55, 0.65]);
        }
        _ => {}
    }
    let mut top = hy + 0.45;
    if umbrella && !sitting && !working {
        let ux = -0.24;
        r.part(Vec3::new(ux, hy + 0.15, 0.14), Vec3::new(0.02, 0.45, 0.02), [0.3, 0.25, 0.2]);
        r.part(Vec3::new(ux, hy + 0.62, 0.14), Vec3::new(0.52, 0.05, 0.52), l.umbrella);
        r.part(Vec3::new(ux, hy + 0.7, 0.14), Vec3::new(0.3, 0.04, 0.3), l.umbrella);
        top = hy + 0.95;
    }

    let bubble = Vec3::new(p.pos.x, p.y + (top + 0.25) * l.scale, p.pos.y);
    if let State::Chatting(_) = p.state {
        let slot = ((time - p.chat_start) / 1.7) as i32;
        if (slot % 2 == 0) == p.first_speaker {
            draw_emote(fx, bubble, p.emote, cam_yaw);
        }
    }
    // singers show a music note now and then, each in their own time
    if singing && ((time * 0.6 + unit(p.seed) * 7.0) % 2.0) < 0.9 {
        draw_emote(fx, bubble, Emote::Note, cam_yaw);
    }
}

pub(super) fn draw_emote(fx: &mut Boxes, pos: Vec3, e: Emote, cam_yaw: f32) {
    let mut r = Rig::new(fx, pos, cam_yaw, 1.0);
    r.part_mat(Vec3::ZERO, Vec3::new(0.3, 0.22, 0.03), [1.0, 1.0, 0.97], MAT_UNLIT);
    r.part_mat(Vec3::new(0.08, -0.25, 0.0), Vec3::new(0.06, 0.05, 0.03), [1.0, 1.0, 0.97], MAT_UNLIT);
    let z = 0.04;
    match e {
        Emote::Talk => {
            for x in [-0.13, 0.0, 0.13] {
                r.part_mat(Vec3::new(x, 0.0, z), Vec3::new(0.04, 0.04, 0.02), [0.25, 0.22, 0.25], MAT_UNLIT);
            }
        }
        Emote::Heart => {
            let red = [0.9, 0.2, 0.3];
            r.part_mat(Vec3::new(-0.06, 0.04, z), Vec3::new(0.06, 0.06, 0.02), red, MAT_UNLIT);
            r.part_mat(Vec3::new(0.06, 0.04, z), Vec3::new(0.06, 0.06, 0.02), red, MAT_UNLIT);
            r.part_mat(Vec3::new(0.0, -0.05, z), Vec3::new(0.08, 0.06, 0.02), red, MAT_UNLIT);
        }
        Emote::Note => {
            let blue = [0.25, 0.4, 0.85];
            r.part_mat(Vec3::new(0.04, 0.04, z), Vec3::new(0.02, 0.12, 0.02), blue, MAT_UNLIT);
            r.part_mat(Vec3::new(-0.02, -0.08, z), Vec3::new(0.06, 0.04, 0.02), blue, MAT_UNLIT);
            r.part_mat(Vec3::new(0.09, 0.13, z), Vec3::new(0.05, 0.03, 0.02), blue, MAT_UNLIT);
        }
        Emote::Laugh => {
            let y = [0.95, 0.7, 0.1];
            r.part_mat(Vec3::new(0.0, 0.0, z), Vec3::new(0.12, 0.1, 0.02), y, MAT_UNLIT);
            r.part_mat(Vec3::new(0.0, -0.03, z + 0.01), Vec3::new(0.07, 0.03, 0.02), [0.4, 0.15, 0.1], MAT_UNLIT);
        }
    }
}
