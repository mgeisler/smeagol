use crate::State;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

lazy_static::lazy_static! {
    static ref KEY_COMMANDS: Vec<KeyCommand> = {
        vec![
            KeyCommand {
                keys: vec![Key::Up, Key::Char('k')],
                action: Action::PanUp,
                description: "pan up"
            },
            KeyCommand {
                keys: vec![Key::Down, Key::Char('j')],
                action: Action::PanDown,
                description: "pan down"
            },
            KeyCommand {
                keys: vec![Key::Left, Key::Char('h')],
                action: Action::PanLeft,
                description: "pan left"
            },
            KeyCommand {
                keys: vec![Key::Right, Key::Char('l')],
                action: Action::PanRight,
                description: "pan right"
            },
            KeyCommand {
                keys: vec![Key::Char(' ')],
                action: Action::ToggleSimulation,
                description: "start/stop simulation"
            },
            KeyCommand {
                keys: vec![Key::Char('='), Key::Char('+')],
                action: Action::IncreaseStep,
                description: "increase step size by a factor of 2"
            },
            KeyCommand {
                keys: vec![Key::Char('-'), Key::Char('_')],
                action: Action::DecreaseStep,
                description: "decrease step size by a factor of 2"
            },
            KeyCommand {
                keys: vec![Key::Char('[')],
                action: Action::IncreaseScale,
                description: "zoom out"
            },
            KeyCommand {
                keys: vec![Key::Char(']')],
                action: Action::DecreaseScale,
                description: "zoom in"
            },
            KeyCommand {
                keys: vec![Key::Char('q')],
                action: Action::Quit,
                description: "quit"
            },
        ]
    };
}

#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    Up,
    Down,
    Left,
    Right,
}

impl Key {
    fn into_event(self) -> cursive::event::Event {
        match self {
            Key::Char(c) => cursive::event::Event::Char(c),
            Key::Up => cursive::event::Event::Key(cursive::event::Key::Up),
            Key::Down => cursive::event::Event::Key(cursive::event::Key::Down),
            Key::Left => cursive::event::Event::Key(cursive::event::Key::Left),
            Key::Right => cursive::event::Event::Key(cursive::event::Key::Right),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    IncreaseStep,
    DecreaseStep,
    IncreaseScale,
    DecreaseScale,
    ToggleSimulation,
    Quit,
}

#[derive(Clone, Debug)]
pub struct KeyCommand {
    keys: Vec<Key>,
    action: Action,
    description: &'static str,
}

const MOVEMENT_FACTOR: u64 = 4;
const MIN_SCALE: u64 = 1;
const MAX_SCALE: u64 = 1 << 48;

fn pan_down(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.1 += (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn pan_up(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.1 -= (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn pan_left(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.0 -= (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn pan_right(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.0 += (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn toggle_simulation(is_running: &Arc<AtomicBool>) {
    is_running.store(!is_running.load(Ordering::SeqCst), Ordering::SeqCst);
}

fn increase_scale(scale: &Arc<Mutex<u64>>) {
    let mut scale = scale.lock().unwrap();
    if *scale < MAX_SCALE {
        *scale <<= 1;
    }
}

fn decrease_scale(scale: &Arc<Mutex<u64>>) {
    let mut scale = scale.lock().unwrap();
    if *scale > MIN_SCALE {
        *scale >>= 1;
    }
}

fn increase_step(step: &Arc<Mutex<u64>>) {
    let mut step = step.lock().unwrap();
    if *step < (1 << 63) {
        *step <<= 1;
    }
}

fn decrease_step(step: &Arc<Mutex<u64>>) {
    let mut step = step.lock().unwrap();
    if *step > 1 {
        *step >>= 1;
    }
}

fn quit(siv: &mut cursive::Cursive) {
    siv.quit()
}

pub fn setup_key_commands(siv: &mut cursive::Cursive, state: &State) {
    for key_command in KEY_COMMANDS.iter() {
        for &key in &key_command.keys {
            match key_command.action {
                Action::PanDown => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            pan_down(&state.center, &state.scale)
                        }),
                    );
                }
                Action::PanUp => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            pan_up(&state.center, &state.scale)
                        }),
                    );
                }
                Action::PanLeft => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            pan_left(&state.center, &state.scale)
                        }),
                    );
                }
                Action::PanRight => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            pan_right(&state.center, &state.scale)
                        }),
                    );
                }
                Action::IncreaseScale => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            increase_scale(&state.scale)
                        }),
                    );
                }
                Action::DecreaseScale => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            decrease_scale(&state.scale)
                        }),
                    );
                }
                Action::IncreaseStep => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            increase_step(&state.step)
                        }),
                    );
                }
                Action::DecreaseStep => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            decrease_step(&state.step)
                        }),
                    );
                }
                Action::ToggleSimulation => {
                    siv.add_global_callback(
                        key.into_event(),
                        enclose!((state) move |_: &mut cursive::Cursive| {
                            toggle_simulation(&state.is_running)
                        }),
                    );
                }
                Action::Quit => {
                    siv.add_global_callback(key.into_event(), quit);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_scale() {
        let scale = Arc::new(Mutex::new(8));

        increase_scale(&scale);
        assert_eq!(*scale.lock().unwrap(), 16);

        decrease_scale(&scale);
        assert_eq!(*scale.lock().unwrap(), 8);

        let min_scale = Arc::new(Mutex::new(MIN_SCALE));
        decrease_scale(&min_scale);
        assert_eq!(*min_scale.lock().unwrap(), MIN_SCALE);

        let max_scale = Arc::new(Mutex::new(MAX_SCALE));
        increase_scale(&max_scale);
        assert_eq!(*max_scale.lock().unwrap(), MAX_SCALE);
    }

    #[test]
    fn pan() {
        let center = Arc::new(Mutex::new((0, 0)));
        let scale = Arc::new(Mutex::new(4));

        pan_down(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (0, 4 * MOVEMENT_FACTOR as i64));

        pan_up(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (0, 0));

        pan_right(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (4 * MOVEMENT_FACTOR as i64, 0));

        pan_left(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (0, 0));
    }

    #[test]
    fn dummy_setup_key_commands() {
        let mut siv = cursive::Cursive::dummy();
        let life = smeagol::Life::new();
        let state = State::new_centered(life, 20, 20);
        setup_key_commands(&mut siv, &state);
    }
}
