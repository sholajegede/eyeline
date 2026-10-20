use eyeline_core::Settings;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn create(app: &AppHandle, settings: &Settings) -> tauri::Result<()> {
    let (x, y) = anchor(app, settings);
    let window = WebviewWindowBuilder::new(app, "prompter", WebviewUrl::App("index.html".into()))
        .title("Eyeline Prompter")
        .inner_size(settings.width, settings.height)
        .position(x, y)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .content_protected(settings.hide_from_capture)
        .build()?;

    #[cfg(target_os = "macos")]
    crate::mac::raise_above_menu_bar(app, &window);
    #[cfg(not(target_os = "macos"))]
    let _ = window;
    Ok(())
}

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("prompter") {
        window.show()?;
    }
    Ok(())
}

pub fn toggle_visibility(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("prompter") {
        if window.is_visible()? {
            window.hide()?;
        } else {
            window.show()?;
        }
    }
    Ok(())
}

fn anchor(app: &AppHandle, settings: &Settings) -> (f64, f64) {
    let monitor = app
        .get_webview_window("main")
        .and_then(|w| w.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return (200.0, 0.0);
    };
    let scale = monitor.scale_factor();
    let origin_x = monitor.position().x as f64 / scale;
    let origin_y = monitor.position().y as f64 / scale;
    let screen_w = monitor.size().width as f64 / scale;

    let x = origin_x + ((screen_w - settings.width) / 2.0).max(0.0);
    let y = if settings.mode == "notch" {
        origin_y
    } else {
        origin_y + 120.0
    };
    (x, y)
}
