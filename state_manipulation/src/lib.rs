extern crate rand;
extern crate common;
extern crate regex;

use common::*;
use common::Turn::*;

use regex::Regex;

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
    let rng: StdRng = SeedableRng::from_seed(seed);

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

    let regex = generate_regex(&mut rng);

    State {
        rng: rng,
        title_screen: title_screen,
        text: String::new(),
        regex,
        examples: Vec::new(),
        guessed_regex: Regex::new("").unwrap(),
        turn: InProgress,
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
    let mut enter_key = false;

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
            Event::KeyReleased {
                key: KeyCode::Enter,
                ctrl: _,
                shift: _,
            } => {
                enter_key = true;
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

    let backspace_spec = ButtonSpec {
        x: 20 + (4 * 10),
        y: 20,
        w: 7,
        h: 3,
        text: "⌫".to_string(),
        id: 10,
    };

    if do_button(platform,
                 &mut state.ui_context,
                 &backspace_spec,
                 left_mouse_pressed,
                 left_mouse_released) || backspace_key {
        state.text.pop();
    }
    let enter_spec = ButtonSpec {
        x: 18 + (4 * 10),
        y: 25,
        w: 11,
        h: 3,
        text: "Submit".to_string(),
        id: 12,
    };

    if do_button(platform,
                 &mut state.ui_context,
                 &enter_spec,
                 left_mouse_pressed,
                 left_mouse_released) || enter_key {
        if state.examples.iter().any(|e| e.text == state.text) {
            //TODO note example was already added
        } else {
            state.examples.push(Example::new(&state.text, &state.regex));

            let mut guessed_regex;

            if state.guessed_regex.as_str().is_empty() {
                guessed_regex = state
                    .examples
                    .iter()
                    .fold(String::new(), |mut acc, ex| {
                        if ex.matched {
                            if !acc.is_empty() {
                                acc.push('|');
                            };
                            acc.push('(');
                            acc.push_str(&ex.text);
                            acc.push(')');

                        };
                        acc
                    });
            } else {
                guessed_regex = state.guessed_regex.as_str().to_owned();

                let mut sub_regexes = get_sub_regexes(&guessed_regex);
                let new_example = state.examples.last().unwrap();
                if new_example.matched {
                    //extend a regex to make the new example match
                    for s in sub_regexes.iter_mut() {
                        if let Some(extended) = extend_to_fit(s, new_example) {
                            *s = extended;
                            break;
                        }
                    }
                } else {
                    //make sure none of the sub_regexes match the new example
                    for s in sub_regexes.iter_mut() {
                        if let Some(contracted) = contract_to_avoid(s, new_example) {
                            *s = contracted;
                        }
                    }
                }

                guessed_regex = collect_sub_regexes(sub_regexes);
            }

            guessed_regex = remove_parens(&guessed_regex, &state.examples);

            guessed_regex = convert_star_to_plus(&guessed_regex);


            if let Ok(regex) = edged_regex(&guessed_regex) {
                state.guessed_regex = regex;
            } else {
                if cfg!(debug_assertions) {
                    println!("bad guess: {}", guessed_regex);
                }
            }
        }
    }

    (platform.print_xy)(20,
                        5,
                        state.regex.as_str().trim_matches(|c| c == '^' || c == '$'));
    (platform.print_xy)(20,
                        7,
                        state
                            .guessed_regex
                            .as_str()
                            .trim_matches(|c| c == '^' || c == '$'));

    let current_example = Example::new(&state.text, &state.regex);

    current_example.print_xy(platform, 7, 10);


    //TODO pagination/scrolling
    for (index, e) in state.examples.iter().enumerate() {
        let i = index as i32;

        e.print_xy(platform, 50, (2 * i) + 3);
    }

    match state.turn {
        InProgress => {
            //TODO fuzzy matching or keep simplifying?
            if state.regex.as_str() == state.guessed_regex.as_str() {
                state.turn = Finished;
            }
        }
        Finished => {
            (platform.print_xy)(20, 15, "They figured it out!");

            let new_spec = ButtonSpec {
                x: 45,
                y: 14,
                w: 12,
                h: 3,
                text: "New Puzzle".to_string(),
                id: 220,
            };

            if do_button(platform,
                         &mut state.ui_context,
                         &new_spec,
                         left_mouse_pressed,
                         left_mouse_released) {
                *state = make_state((platform.size)(), false, state.rng);
            }
        }
    }

    false
}


fn convert_star_to_plus(regex: &str) -> String {
    let classes_and_chars = split_into_classes_and_chars(regex);

    let mut result = String::new();

    if classes_and_chars.len() >= 3 {
        let mut windows = classes_and_chars.windows(3).peekable();

        while let Some(w) = windows.next() {
            if w[0] == w[1] && w[2].chars().nth(0) == Some('*') {
                result.push_str(w[0]);
                result.push('+');

                windows.next();
                windows.next();
            } else {
                result.push_str(w[0]);

                if windows.peek().is_none() {
                    result.push_str(w[1]);
                    result.push_str(w[2]);
                }
            }
        }
    } else {
        result.push_str(regex);
    }

    result
}

#[cfg(test)]
mod convert_star_to_plus {
    use super::convert_star_to_plus;
    #[test]
    fn minimal() {
        assert_eq!("", convert_star_to_plus(""));
    }
    #[test]
    fn one_digit() {
        assert_eq!("0", convert_star_to_plus("0"));
        assert_eq!("1", convert_star_to_plus("1"));
        assert_eq!("2", convert_star_to_plus("2"));
        assert_eq!("3", convert_star_to_plus("3"));
    }
    #[test]
    fn one_digit_star_to_plus() {
        assert_eq!("0+", convert_star_to_plus("00*"));
        assert_eq!("1+", convert_star_to_plus("11*"));
        assert_eq!("2+", convert_star_to_plus("22*"));
        assert_eq!("3+", convert_star_to_plus("33*"));
    }
    #[test]
    fn one_digit_plus_to_plus() {
        assert_eq!("0+", convert_star_to_plus("0+"));
        assert_eq!("1+", convert_star_to_plus("1+"));
        assert_eq!("2+", convert_star_to_plus("2+"));
        assert_eq!("3+", convert_star_to_plus("3+"));
    }
}

fn split_into_classes_and_chars(regex: &str) -> Vec<&str> {
    let mut spilt_indices = Vec::new();

    let mut split = true;

    for (i, c) in regex.chars().enumerate() {
        if c == '[' {
            split = false;
        }

        if c == ']' {
            split = true;
        }

        if split {
            spilt_indices.push(i);
        }
    }

    let mut result = Vec::new();

    let mut last_index = 0;

    if spilt_indices.len() >= 2 {
        for indices in spilt_indices.windows(2) {
            result.push(unsafe { regex.slice_unchecked(indices[0], indices[1]) });

            last_index = indices[1];
        }
    }

    result.push(regex.split_at(last_index).1);

    result
}

fn remove_parens(regex: &str, _examples: &Vec<Example>) -> String {
    let len = regex.len();

    let mut removal_indicies = Vec::new();

    for (left, _) in regex.match_indices('(') {
        if let Some(right) = get_matching_paren_index(regex, left) {
            if regex
                   .chars()
                   .nth(right + 1)
                   .map(|c| c != '*' && c != '+')
                   .unwrap_or(true) {
                removal_indicies.push(left);
                if right < len {
                    removal_indicies.push(right);
                }
            } else {
                //TODO remove the parens and the times and see if the result matches
                // the examples
                // if matches_examples(edged_regex(removed), examples) {
                //
                // }
            }
        }
    }

    let mut result = String::from(regex);

    removal_indicies.sort();

    for i in removal_indicies.iter().rev() {
        result.remove(*i);
    }

    result
}

fn get_matching_paren_index(s: &str, left_index: usize) -> Option<usize> {
    let mut right_index = left_index;
    let mut counter = 1;
    for c in s.chars().skip(left_index + 1) {
        right_index += 1;

        if c == '(' {
            counter += 1;
        } else if c == ')' {
            counter -= 1;
        }

        if counter <= 0 {
            return Some(right_index);
        }
    }

    None
}

#[cfg(test)]
mod get_matching_paren_index {
    use super::get_matching_paren_index;

    #[test]
    fn minimal_none() {
        assert_eq!(None, get_matching_paren_index("", 0));
        assert_eq!(None, get_matching_paren_index("(", 0));
    }

    #[test]
    fn minimal_find() {
        assert_eq!(Some(1), get_matching_paren_index("()", 0));
    }

    #[test]
    fn inner() {
        assert_eq!(Some(3), get_matching_paren_index("1(2)3", 1));
    }

    #[test]
    fn nested() {
        assert_eq!(Some(4), get_matching_paren_index("(1(2)3)", 2));
        assert_eq!(Some(6), get_matching_paren_index("(1(2)3)", 0));
    }
}

fn get_sub_regexes(regex: &str) -> Vec<String> {
    let mut inner_regex = if regex.starts_with('^') {
        regex.split_at(1).1
    } else {
        regex
    };

    inner_regex = if inner_regex.ends_with('$') {
        inner_regex.split_at(inner_regex.len() - 1).0
    } else {
        inner_regex
    };

    inner_regex.split("|").map(String::from).collect()
}

fn extend_to_fit(regex_str: &str, example: &Example) -> Option<String> {
    if let Ok(regex) = edged_regex(regex_str) {
        if regex.is_match(&example.text) {
            None
        } else {
            //TODO handle more cases
            let mut try = format!("({}+)", regex_str);
            println!("{}", try);

            if edged_regex(&try)
                   .map(|r| r.is_match(&example.text))
                   .unwrap_or(false) {
                return Some(try);
            }

            try = format!("({}*)", regex_str);

            if edged_regex(&try)
                   .map(|r| r.is_match(&example.text))
                   .unwrap_or(false) {
                return Some(try);
            }

            if cfg!(debug_assertions) {
                println!("default extension");
            }

            Some(format!("{}|{}", regex_str, example.text))
        }
    } else {
        Some(String::from(".*"))
    }
}
fn contract_to_avoid(regex_str: &str, example: &Example) -> Option<String> {
    if let Ok(regex) = edged_regex(regex_str) {
        if regex.is_match(&example.text) {
            //TODO handle more cases
            let len = example.text.len();

            if example
                   .text
                   .chars()
                   .nth(len - 2)
                   .map(|c| c == '*' || c == '+')
                   .unwrap_or(false) {
                let mut try = example.text.to_owned();
                try.remove(len - 2);
                if edged_regex(&try)
                       .map(|r| !r.is_match(&example.text))
                       .unwrap_or(false) {
                    return Some(try);
                }
            }

            None
        } else {
            None
        }
    } else {
        Some(String::from(""))
    }
}
fn collect_sub_regexes(sub_regexes: Vec<String>) -> String {
    let mut result = String::new();

    let mut not_first = false;
    for s in sub_regexes.iter() {
        if not_first {
            result.push('|');
        } else {
            not_first = true;
        }
        result.push_str(s);
    }

    result
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
