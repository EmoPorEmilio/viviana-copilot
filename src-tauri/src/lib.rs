// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_cursor() -> Result<(i32, i32), String> {
    // read the global cursor position and return as (x, y)
    use device_query::{DeviceQuery, DeviceState};
    let device_state = DeviceState::new();
    let mouse = device_state.get_mouse();
    Ok((mouse.coords.0, mouse.coords.1))
}

#[tauri::command]
fn move_cursor(dx: i32, dy: i32) -> Result<(i32, i32), String> {
    // read current cursor, compute new position, and set it
    use device_query::{DeviceQuery, DeviceState};
    use enigo::{Enigo, MouseControllable};

    let device_state = DeviceState::new();
    let mouse = device_state.get_mouse();
    // Use relative movement to avoid DPI/coordinate space mismatches
    let mut enigo = Enigo::new();
    enigo.mouse_move_relative(dx, dy);

    // Query the new position after moving
    let mouse_after = device_state.get_mouse();
    Ok((mouse_after.coords.0, mouse_after.coords.1))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_cursor, move_cursor])
        .setup(|_app| {
            // Spawn a background thread that polls global keyboard state and moves the cursor
            // when H/J/K/L are pressed. This works even if the app window is inactive.
            std::thread::spawn(|| {
                use device_query::{DeviceQuery, DeviceState, Keycode};
                use enigo::{Enigo, MouseControllable};

                use std::time::{Duration, Instant};

                // timings for key repeat
                let initial_delay = Duration::from_millis(300);
                let repeat_delay = Duration::from_millis(50);

                let device_state = DeviceState::new();
                // small vector of pressed keys -> (key, last_trigger_instant, is_first)
                let mut key_states: Vec<(Keycode, Instant, bool)> = Vec::new();

                loop {
                    let keys = device_state.get_keys();
                    let now = Instant::now();

                    // handle currently pressed keys
                    for k in keys.iter() {
                        // handle H/J/K/L and special keys 0 and Shift+4 ($)
                        // find existing state index
                        if let Some(pos) = key_states.iter().position(|(kk, _, _)| kk == k) {
                            let (_, last, is_first) = &mut key_states[pos];
                            if *is_first {
                                if now.duration_since(*last) >= initial_delay {
                                    let mut enigo = Enigo::new();
                                    match k {
                                        Keycode::H => enigo.mouse_move_relative(-8, 0),
                                        Keycode::J => enigo.mouse_move_relative(0, 8),
                                        Keycode::K => enigo.mouse_move_relative(0, -8),
                                        Keycode::L => enigo.mouse_move_relative(8, 0),
                                        Keycode::Key0 | Keycode::Numpad0 => {
                                            let (_, y) = {
                                                let ds = DeviceState::new();
                                                let m = ds.get_mouse();
                                                (m.coords.0, m.coords.1)
                                            };
                                            enigo.mouse_move_to(0, y);
                                        }
                                        Keycode::Key4 | Keycode::Numpad4 => {
                                            // detect shift pressed -> '$'
                                            if keys.contains(&Keycode::LShift)
                                                || keys.contains(&Keycode::RShift)
                                            {
                                                #[cfg(target_os = "windows")]
                                                {
                                                    use winapi::um::winuser::{
                                                        GetSystemMetrics, SM_CXSCREEN,
                                                    };
                                                    let max_x =
                                                        unsafe { GetSystemMetrics(SM_CXSCREEN) }
                                                            - 1;
                                                    let (_, y) = {
                                                        let ds = DeviceState::new();
                                                        let m = ds.get_mouse();
                                                        (m.coords.0, m.coords.1)
                                                    };
                                                    enigo.mouse_move_to(max_x, y);
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                    *is_first = false;
                                    *last = now;
                                }
                            } else {
                                if now.duration_since(*last) >= repeat_delay {
                                    let mut enigo = Enigo::new();
                                    match k {
                                        Keycode::H => enigo.mouse_move_relative(-8, 0),
                                        Keycode::J => enigo.mouse_move_relative(0, 8),
                                        Keycode::K => enigo.mouse_move_relative(0, -8),
                                        Keycode::L => enigo.mouse_move_relative(8, 0),
                                        _ => {}
                                    }
                                    *last = now;
                                }
                            }
                        } else {
                            // newly pressed: trigger immediately and record
                            let mut enigo = Enigo::new();
                            match k {
                                Keycode::H => enigo.mouse_move_relative(-8, 0),
                                Keycode::J => enigo.mouse_move_relative(0, 8),
                                Keycode::K => enigo.mouse_move_relative(0, -8),
                                Keycode::L => enigo.mouse_move_relative(8, 0),
                                Keycode::Key0 | Keycode::Numpad0 => {
                                    let (_, y) = {
                                        let ds = DeviceState::new();
                                        let m = ds.get_mouse();
                                        (m.coords.0, m.coords.1)
                                    };
                                    enigo.mouse_move_to(0, y);
                                }
                                Keycode::Key4 | Keycode::Numpad4 => {
                                    if keys.contains(&Keycode::LShift)
                                        || keys.contains(&Keycode::RShift)
                                    {
                                        #[cfg(target_os = "windows")]
                                        {
                                            use winapi::um::winuser::{
                                                GetSystemMetrics, SM_CXSCREEN,
                                            };
                                            let max_x =
                                                unsafe { GetSystemMetrics(SM_CXSCREEN) } - 1;
                                            let (_, y) = {
                                                let ds = DeviceState::new();
                                                let m = ds.get_mouse();
                                                (m.coords.0, m.coords.1)
                                            };
                                            enigo.mouse_move_to(max_x, y);
                                        }
                                    }
                                }
                                _ => {}
                            }
                            key_states.push(((*k).clone(), now, true));
                        }
                    }

                    // remove released keys from state
                    // remove released keys from state
                    let prev_keys: Vec<Keycode> =
                        key_states.iter().map(|(kk, _, _)| kk.clone()).collect();
                    for k in prev_keys.iter() {
                        if !keys.contains(k) {
                            if let Some(pos) = key_states.iter().position(|(kk, _, _)| kk == k) {
                                key_states.remove(pos);
                            }
                        }
                    }

                    std::thread::sleep(Duration::from_millis(10));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
