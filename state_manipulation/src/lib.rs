extern crate rand;
extern crate common;

use common::*;

use rand::{StdRng, SeedableRng, Rng};

//NOTE(Ryan1729): debug_assertions only appears to work correctly when the
//crate is not a dylib. Assuming you make this crate *not* a dylib on release,
//these configs should work
#[cfg(debug_assertions)]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //skip the title screen
    println!("debug on");

    let seed: &[_] = &[42];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    make_state(size, false, rng)
}
#[cfg(not(debug_assertions))]
#[no_mangle]
pub fn new_state(size: Size) -> State {
    //show the title screen
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|dur| dur.as_secs())
        .unwrap_or(42);

    println!("{}", timestamp);
    let seed: &[_] = &[timestamp as usize];
    let rng: StdRng = SeedableRng::from_seed(seed);

    make_state(size, true, rng)
}


fn make_state(size: Size, title_screen: bool, mut rng: StdRng) -> State {
    let mut row = Vec::new();

    for _ in 0..size.width {
        row.push(rng.gen::<u8>());
    }

    State {
        rng: rng,
        title_screen: title_screen,
        text: String::new(),
        ui_context: UIContext::new(),
    }
}

#[no_mangle]
//returns true if quit requested
pub fn update_and_render(platform: &Platform, state: &mut State, events: &mut Vec<Event>) -> bool {
    if state.title_screen {

        for event in events {
            cross_mode_event_handling(platform, state, event);
            match *event {
                Event::Close |
                Event::KeyPressed {
                    key: KeyCode::Escape,
                    ctrl: _,
                    shift: _,
                } => return true,
                Event::KeyPressed {
                    key: _,
                    ctrl: _,
                    shift: _,
                } => state.title_screen = false,
                _ => (),
            }
        }

        false
    } else {
        game_update_and_render(platform, state, events)
    }
}

pub fn game_update_and_render(platform: &Platform,
                              state: &mut State,
                              events: &mut Vec<Event>)
                              -> bool {
    let mut left_mouse_pressed = false;
    let mut left_mouse_released = false;

    let mut backspace_key = false;

    let mut num_key = [false, false, false, false];

    for event in events {
        cross_mode_event_handling(platform, state, event);

        match *event {
            Event::KeyPressed {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_pressed = true;
            }
            Event::KeyReleased {
                key: KeyCode::MouseLeft,
                ctrl: _,
                shift: _,
            } => {
                left_mouse_released = true;
            }
            Event::Close |
            Event::KeyPressed {
                key: KeyCode::Escape,
                ctrl: _,
                shift: _,
            } => return true,
            Event::KeyReleased {
                key: KeyCode::Row0,
                ctrl: _,
                shift: _,
            } |
            Event::KeyReleased {
                key: KeyCode::Num0,
                ctrl: _,
                shift: _,
            } => {
                num_key[0] = true;
            }
            Event::KeyReleased {
                key: KeyCode::Row1,
                ctrl: _,
                shift: _,
            } |
            Event::KeyReleased {
                key: KeyCode::Num1,
                ctrl: _,
                shift: _,
            } => {
                num_key[1] = true;
            }
            Event::KeyReleased {
                key: KeyCode::Row2,
                ctrl: _,
                shift: _,
            } |
            Event::KeyReleased {
                key: KeyCode::Num2,
                ctrl: _,
                shift: _,
            } => {
                num_key[2] = true;
            }
            Event::KeyReleased {
                key: KeyCode::Row3,
                ctrl: _,
                shift: _,
            } |
            Event::KeyReleased {
                key: KeyCode::Num3,
                ctrl: _,
                shift: _,
            } => {
                num_key[3] = true;
            }
            Event::KeyReleased {
                key: KeyCode::Backspace,
                ctrl: _,
                shift: _,
            } => {
                backspace_key = true;
            }
            _ => (),
        }
    }

    state.ui_context.frame_init();

    for index in 0..4 {
        let i = index as i32;

        let spec = ButtonSpec {
            x: 20 + (i * 10),
            y: 20,
            w: 5,
            h: 3,
            text: index.to_string(),
            id: i + 1,
        };

        if do_button(platform,
                     &mut state.ui_context,
                     &spec,
                     left_mouse_pressed,
                     left_mouse_released) || num_key[index] {
            state.text.push_str(&index.to_string());
        }
    }

    let spec = ButtonSpec {
        x: 20 + (4 * 10),
        y: 20,
        w: 7,
        h: 3,
        text: "⌫".to_string(),
        id: 10,
    };

    if do_button(platform,
                 &mut state.ui_context,
                 &spec,
                 left_mouse_pressed,
                 left_mouse_released) || backspace_key {
        state.text.pop();
    }


    (platform.print_xy)(10, 10, &state.text);

    false
}

fn cross_mode_event_handling(platform: &Platform, state: &mut State, event: &Event) {
    match *event {
        Event::KeyPressed {
            key: KeyCode::R,
            ctrl: true,
            shift: _,
        } => {
            println!("reset");
            *state = new_state((platform.size)());
        }
        _ => (),
    }
}

pub struct ButtonSpec {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub text: String,
    pub id: i32,
}

//calling this once will swallow multiple clicks on the button. We could either
//pass in and return the number of clicks to fix that, or this could simply be
//called multiple times per frame (once for each click).
fn do_button(platform: &Platform,
             context: &mut UIContext,
             spec: &ButtonSpec,
             left_mouse_pressed: bool,
             left_mouse_released: bool)
             -> bool {
    let mut result = false;

    let mouse_pos = (platform.mouse_position)();
    let inside = inside_rect(mouse_pos, spec.x, spec.y, spec.w, spec.h);
    let id = spec.id;

    if context.active == id {
        if left_mouse_released {
            result = context.hot == id && inside;

            context.set_not_active();
        }
    } else if context.hot == id {
        if left_mouse_pressed {
            context.set_active(id);
        }
    }

    if inside {
        context.set_next_hot(id);
    }

    if context.active == id && left_mouse_pressed {
        draw_rect_with(platform,
                       spec.x,
                       spec.y,
                       spec.w,
                       spec.h,
                       ["╔", "═", "╕", "║", "│", "╙", "─", "┘"]);
    } else if context.hot == id {
        draw_rect_with(platform,
                       spec.x,
                       spec.y,
                       spec.w,
                       spec.h,
                       ["┌", "─", "╖", "│", "║", "╘", "═", "╝"]);
    } else {
        draw_rect(platform, spec.x, spec.y, spec.w, spec.h);
    }

    print_centered_line(platform, spec.x, spec.y, spec.w, spec.h, &spec.text);

    return result;
}

pub fn inside_rect(point: Point, x: i32, y: i32, w: i32, h: i32) -> bool {
    x <= point.x && y <= point.y && point.x < x + w && point.y < y + h
}

fn print_centered_line(platform: &Platform, x: i32, y: i32, w: i32, h: i32, text: &str) {
    let x_ = {
        let rect_middle = x + (w / 2);

        rect_middle - (text.chars().count() as f32 / 2.0) as i32
    };

    let y_ = y + (h / 2);

    (platform.print_xy)(x_, y_, &text);
}


fn draw_rect(platform: &Platform, x: i32, y: i32, w: i32, h: i32) {
    draw_rect_with(platform,
                   x,
                   y,
                   w,
                   h,
                   ["┌", "─", "┐", "│", "│", "└", "─", "┘"]);
}

fn draw_double_line_rect(platform: &Platform, x: i32, y: i32, w: i32, h: i32) {
    draw_rect_with(platform,
                   x,
                   y,
                   w,
                   h,
                   ["╔", "═", "╗", "║", "║", "╚", "═", "╝"]);
}

fn draw_rect_with(platform: &Platform, x: i32, y: i32, w: i32, h: i32, edges: [&str; 8]) {
    (platform.clear)(Some(Rect::from_values(x, y, w, h)));

    let right = x + w - 1;
    let bottom = y + h - 1;
    // top
    (platform.print_xy)(x, y, edges[0]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, y, edges[1]);
    }
    (platform.print_xy)(right, y, edges[2]);

    // sides
    for i in (y + 1)..bottom {
        (platform.print_xy)(x, i, edges[3]);
        (platform.print_xy)(right, i, edges[4]);
    }

    //bottom
    (platform.print_xy)(x, bottom, edges[5]);
    for i in (x + 1)..right {
        (platform.print_xy)(i, bottom, edges[6]);
    }
    (platform.print_xy)(right, bottom, edges[7]);
}
