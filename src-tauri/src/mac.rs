use objc2::{msg_send, runtime::AnyObject};
use tauri::{AppHandle, WebviewWindow};

const STATUS_WINDOW_LEVEL: isize = 25;
const CAN_JOIN_ALL_SPACES: usize = 1 << 0;
const STATIONARY: usize = 1 << 4;
const IGNORES_CYCLE: usize = 1 << 6;
const FULL_SCREEN_AUXILIARY: usize = 1 << 8;

pub fn raise_above_menu_bar(app: &AppHandle, window: &WebviewWindow) {
    let Ok(ptr) = window.ns_window() else {
        return;
    };
    let address = ptr as usize;
    let _ = app.run_on_main_thread(move || unsafe {
        let ns_window = address as *mut AnyObject;
        let behavior: usize =
            CAN_JOIN_ALL_SPACES | STATIONARY | IGNORES_CYCLE | FULL_SCREEN_AUXILIARY;
        let _: () = msg_send![ns_window, setLevel: STATUS_WINDOW_LEVEL];
        let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
    });
}
