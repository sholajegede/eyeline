use tauri::{plugin::TauriPlugin, Emitter, Wry};
use tauri_plugin_global_shortcut::{Builder, Code, Modifiers, Shortcut, ShortcutState};

const MODS: Modifiers = Modifiers::CONTROL.union(Modifiers::ALT);

fn bindings() -> [(Shortcut, &'static str); 6] {
    [
        (Shortcut::new(Some(MODS), Code::Space), "toggle"),
        (Shortcut::new(Some(MODS), Code::ArrowUp), "faster"),
        (Shortcut::new(Some(MODS), Code::ArrowDown), "slower"),
        (Shortcut::new(Some(MODS), Code::ArrowRight), "next-cue"),
        (Shortcut::new(Some(MODS), Code::ArrowLeft), "previous-cue"),
        (Shortcut::new(Some(MODS), Code::KeyP), "visibility"),
    ]
}

pub fn plugin() -> TauriPlugin<Wry> {
    let table = bindings();
    let shortcuts: Vec<Shortcut> = table.iter().map(|(s, _)| *s).collect();

    Builder::new()
        .with_shortcuts(shortcuts)
        .expect("hotkeys are valid")
        .with_handler(move |app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            let Some((_, action)) = table.iter().find(|(s, _)| s == shortcut) else {
                return;
            };
            if *action == "visibility" {
                let _ = crate::prompter::toggle_visibility(app);
            } else {
                let _ = app.emit("prompter-action", *action);
            }
        })
        .build()
}
