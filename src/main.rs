//! Meeple: an ambient, procedurally generated isometric voxel world.

mod app;
mod camera;
mod names;
mod quality;
mod render;
mod sim;
mod util;
mod noise4;
mod world;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent, TouchEvent, WebGl2RenderingContext, WheelEvent};

use app::{App, Opts};

/// `?quality=auto|high|medium|low|0..5`. Auto (the default) adapts to frame
/// times, starting a little lower on TV browsers; the others pin a tier.
fn quality(w: &web_sys::Window, q: Option<String>) -> (u32, bool) {
    match q.as_deref().map(str::to_lowercase).as_deref() {
        Some("high") => (0, false),
        Some("medium") => (2, false),
        Some("low") => (4, false),
        Some(n) if n.parse::<u32>().is_ok() => (n.parse().unwrap(), false),
        _ => {
            let ua = w.navigator().user_agent().unwrap_or_default().to_lowercase();
            let tv = ["tv", "philips", "titan", "tizen", "webos", "crkey"].iter().any(|k| ua.contains(k));
            (if tv { 1 } else { 0 }, true)
        }
    }
}

fn opts() -> Opts {
    let w = web_sys::window().unwrap();
    let search = w.location().search().unwrap_or_default();
    let q = web_sys::UrlSearchParams::new_with_str(&search).ok();
    let get = |k: &str| q.as_ref().and_then(|q| q.get(k));
    let seed = get("seed")
        .and_then(|s| s.parse::<u32>().ok().or_else(|| Some(util::hash_u32(s.bytes().fold(7, |a, b| a * 31 + b as u32)))))
        .unwrap_or_else(|| (js_sys::Math::random() * 1_000_000.0) as u32);
    let season = get("season").and_then(|s| match s.to_lowercase().as_str() {
        "spring" => Some(0.15),
        "summer" => Some(1.15),
        "autumn" | "fall" => Some(2.15),
        "winter" => Some(3.15),
        _ => s.parse().ok(),
    });
    Opts {
        seed,
        quality: quality(&w, get("quality")),
        fx_off: get("fxoff").unwrap_or_default(),
        pixel: get("pixel").and_then(|s| s.parse().ok()).unwrap_or(2).clamp(1, 6),
        day_len: get("day").and_then(|s| s.parse().ok()).unwrap_or(1200.0f64).max(30.0),
        season,
        hour: get("hour").and_then(|s| s.parse().ok()).unwrap_or(7.5),
        at: get("x").zip(get("z")).and_then(|(x, z)| Some((x.parse().ok()?, z.parse().ok()?))),
        bella: get("bella").is_some(),
        t: get("t").and_then(|s| s.parse().ok()),
        view: get("yaw").zip(get("zoom")).and_then(|(y, z)| Some((y.parse().ok()?, z.parse::<f32>().ok()?.clamp(8.0, 130.0)))),
        prof: get("prof").is_some(),
        pitch: get("pitch").and_then(|s| s.parse::<f32>().ok()).map(|p| p.clamp(0.15, 1.2)),
        aurora: get("aurora").map(|s| s.parse().unwrap_or(1.0f32).clamp(0.0, 1.0)),
        auto: !matches!(get("auto").as_deref(), Some("0" | "off" | "false" | "no")),
        resume: get("hour").is_none() && get("season").is_none(),
    }
}

thread_local! {
    /// Every listener this instance added, so a newer build can take over the
    /// page: see `generation`.
    static LISTENERS: RefCell<Vec<(web_sys::EventTarget, String, js_sys::Function, Box<dyn std::any::Any>)>> = RefCell::new(Vec::new());
}

/// A counter on `window`, bumped by each instance as it starts. When the page
/// swaps in a newer build in place (see web/update.js) the older instance sees
/// the count has moved on, stops its loop and lets go of its listeners.
fn generation() -> u32 {
    let w = web_sys::window().unwrap();
    js_sys::Reflect::get(&w, &"__meepleGen".into()).ok().and_then(|v| v.as_f64()).unwrap_or(0.0) as u32
}

fn listen<E: wasm_bindgen::convert::FromWasmAbi + 'static>(
    target: &web_sys::EventTarget,
    name: &str,
    mut f: impl FnMut(E) + 'static,
) {
    let c = Closure::<dyn FnMut(E)>::new(move |e: E| f(e));
    target.add_event_listener_with_callback(name, c.as_ref().unchecked_ref()).unwrap();
    let func: js_sys::Function = c.as_ref().unchecked_ref::<js_sys::Function>().clone();
    LISTENERS.with(|l| l.borrow_mut().push((target.clone(), name.to_string(), func, Box::new(c))));
}

fn retire() {
    LISTENERS.with(|l| {
        for (t, name, func, _closure) in l.borrow_mut().drain(..) {
            let _ = t.remove_event_listener_with_callback(&name, &func);
        }
    });
}

/// The fingers currently on the screen, in CSS pixels.
fn touch_points(e: &TouchEvent) -> Vec<(f64, f64)> {
    let l = e.touches();
    (0..l.length()).filter_map(|i| l.get(i)).map(|t| (t.client_x() as f64, t.client_y() as f64)).collect()
}

/// Where a touch gesture was last frame: the fingers' midpoint and spread.
#[derive(Clone, Copy)]
struct Gesture {
    mid: (f64, f64),
    spread: f64,
    fingers: usize,
}

impl Gesture {
    fn of(p: &[(f64, f64)]) -> Option<Self> {
        if p.is_empty() {
            return None;
        }
        let n = p.len() as f64;
        let mid = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
        let spread = if p.len() > 1 { ((p[0].0 - p[1].0).powi(2) + (p[0].1 - p[1].1).powi(2)).sqrt() } else { 0.0 };
        Some(Gesture { mid, spread, fingers: p.len() })
    }
}

fn main() {
    console_error_panic_hook::set_once();
    let window = web_sys::window().unwrap();
    let gen = generation() + 1;
    let _ = js_sys::Reflect::set(&window, &"__meepleGen".into(), &(gen as f64).into());
    let doc = window.document().unwrap();
    let canvas: HtmlCanvasElement = doc.get_element_by_id("meeple").unwrap().dyn_into().unwrap();
    let attrs = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&attrs, &"antialias".into(), &false.into());
    let _ = js_sys::Reflect::set(&attrs, &"powerPreference".into(), &"high-performance".into());
    let gl: WebGl2RenderingContext = match canvas.get_context_with_context_options("webgl2", &attrs) {
        Ok(Some(c)) => c.dyn_into().unwrap(),
        _ => {
            if let Some(b) = doc.body() {
                b.set_inner_html("<p style='padding:2em'>Meeple needs WebGL2, which this browser doesn't provide.</p>");
            }
            return;
        }
    };

    let opts = opts();
    if window.location().search().unwrap_or_default().contains("bench") {
        let perf = window.performance().unwrap();
        web_sys::console::log_1(&noise4::bench(opts.seed, &|| perf.now()).into());
    }
    let app = match App::new(canvas.clone(), gl, doc.clone(), &opts) {
        Ok(a) => Rc::new(RefCell::new(a)),
        Err(e) => {
            web_sys::console::error_1(&format!("Meeple failed to start: {e}").into());
            return;
        }
    };

    let size = {
        let window = window.clone();
        move || {
            (
                window.inner_width().unwrap().as_f64().unwrap_or(1280.0),
                window.inner_height().unwrap().as_f64().unwrap_or(800.0),
                window.device_pixel_ratio(),
            )
        }
    };
    let (w, h, dpr) = size();
    app.borrow_mut().resize(w, h, dpr);
    // the starting tier's settings (shadows, HDR, chunkiness)
    app.borrow_mut().apply_quality();

    {
        let app = app.clone();
        listen::<web_sys::Event>(&window, "resize", move |_| {
            let (w, h, dpr) = size();
            app.borrow_mut().resize(w, h, dpr);
        });
    }
    {
        let app = app.clone();
        listen::<KeyboardEvent>(&window, "keydown", move |e| {
            let k = e.key();
            let mut a = app.borrow_mut();
            if !a.input.keys.contains(&k) {
                a.key(&k);
            }
            a.input.keys.insert(k);
        });
    }
    {
        let app = app.clone();
        listen::<KeyboardEvent>(&window, "keyup", move |e| {
            app.borrow_mut().input.keys.remove(&e.key());
        });
    }
    {
        let app = app.clone();
        listen::<web_sys::Event>(&window, "blur", move |_| {
            app.borrow_mut().input.keys.clear();
        });
    }
    {
        let app = app.clone();
        listen::<MouseEvent>(&canvas, "mousedown", move |e| {
            let mut a = app.borrow_mut();
            let at = (e.client_x() as f64, e.client_y() as f64);
            a.input.drag = Some(at);
            a.input.down = Some(at);
        });
    }
    {
        let app = app.clone();
        listen::<MouseEvent>(&window, "mousemove", move |e| {
            let mut a = app.borrow_mut();
            a.input.moved_t = 0.0;
            if let Some((x, y)) = a.input.drag {
                if e.buttons() & 1 == 0 {
                    a.input.drag = None;
                    return;
                }
                let (nx, ny) = (e.client_x() as f64, e.client_y() as f64);
                a.drag(nx - x, ny - y);
                a.input.drag = Some((nx, ny));
            }
        });
    }
    {
        let app = app.clone();
        listen::<MouseEvent>(&window, "mouseup", move |e| {
            let mut a = app.borrow_mut();
            a.input.drag = None;
            // a click, not a drag: perhaps on a villager
            let (x, y) = (e.client_x() as f64, e.client_y() as f64);
            if let Some((dx, dy)) = a.input.down.take() {
                if (dx - x).abs() + (dy - y).abs() < 6.0 {
                    a.click(x, y);
                }
            }
        });
    }
    {
        let app = app.clone();
        listen::<WheelEvent>(&canvas, "wheel", move |e| {
            e.prevent_default();
            let mut a = app.borrow_mut();
            a.input.wheel += e.delta_y() as f32;
            a.input.moved_t = 0.0;
        });
    }

    // touch: one finger pans, two pan and pinch to zoom, a tap is a click
    let gesture: Rc<RefCell<Option<Gesture>>> = Rc::new(RefCell::new(None));
    {
        let (app, gesture) = (app.clone(), gesture.clone());
        listen::<TouchEvent>(&canvas, "touchstart", move |e| {
            e.prevent_default();
            let p = touch_points(&e);
            let mut a = app.borrow_mut();
            a.input.moved_t = 0.0;
            a.input.down = if p.len() == 1 && gesture.borrow().is_none() { Some(p[0]) } else { None };
            *gesture.borrow_mut() = Gesture::of(&p);
        });
    }
    {
        let (app, gesture) = (app.clone(), gesture.clone());
        listen::<TouchEvent>(&canvas, "touchmove", move |e| {
            e.prevent_default();
            let p = touch_points(&e);
            let now = Gesture::of(&p);
            let mut a = app.borrow_mut();
            a.input.moved_t = 0.0;
            if let (Some(was), Some(g)) = (*gesture.borrow(), now) {
                if was.fingers == g.fingers {
                    a.drag(g.mid.0 - was.mid.0, g.mid.1 - was.mid.1);
                    if g.fingers > 1 && was.spread > 1.0 && g.spread > 1.0 {
                        a.input.wheel += ((was.spread / g.spread).ln() / 0.0012) as f32;
                    }
                }
            }
            if let (Some((dx, dy)), Some(&(x, y))) = (a.input.down, p.first()) {
                if (dx - x).abs() + (dy - y).abs() >= 10.0 {
                    a.input.down = None;
                }
            }
            *gesture.borrow_mut() = now;
        });
    }
    for name in ["touchend", "touchcancel"] {
        let (app, gesture) = (app.clone(), gesture.clone());
        let tap = name == "touchend";
        listen::<TouchEvent>(&canvas, name, move |e| {
            e.prevent_default();
            let p = touch_points(&e);
            let mut a = app.borrow_mut();
            if p.is_empty() {
                if let (true, Some((x, y))) = (tap, a.input.down.take()) {
                    a.click(x, y);
                }
            }
            a.input.down = None;
            *gesture.borrow_mut() = Gesture::of(&p);
        });
    }

    // animation loop
    let perf = window.performance().unwrap();
    let now = move || perf.now();
    let tick: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let tick2 = tick.clone();
    let mut last: Option<f64> = None;
    let mut ready = false;
    let win = window.clone();
    *tick2.borrow_mut() = Some(Closure::new(move |t: f64| {
        if generation() != gen {
            // a newer build has taken the page: let go of everything
            retire();
            return;
        }
        let cap = app.borrow().min_frame_ms();
        if let (Some(l), true) = (last, cap > 0.0) {
            // slack so a 60 Hz display lands on every second vsync
            if t - l < cap - 4.0 {
                let _ = win.request_animation_frame(tick.borrow().as_ref().unwrap().as_ref().unchecked_ref());
                return;
            }
        }
        let dt = last.map(|l| (t - l) / 1000.0).unwrap_or(1.0 / 60.0) as f32;
        last = Some(t);
        app.borrow_mut().frame(dt, &now);
        if last.is_some() && !ready {
            ready = true;
            // lets the splash screen go
            let _ = js_sys::Reflect::set(&win, &"__meepleReady".into(), &true.into());
        }
        let _ = win.request_animation_frame(tick.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }));
    let _ = window.request_animation_frame(tick2.borrow().as_ref().unwrap().as_ref().unchecked_ref());
}
