# Eyeline

A teleprompter for Mac and Windows that keeps your script right under the camera, so you read and still look at the lens. The prompter window is hidden from screen sharing and recordings.

One-time purchase, no account, and your scripts stay on your machine as plain Markdown files.

## Status

Early development. Working today:

- Script library and editor. Scripts are `.md` files in `Documents/Eyeline` (folders work).
- Cues from `##` headings, stage directions in `[brackets]` that are shown but never read.
- Prompter window under the camera or floating, with countdown, mirror and flip, dimming of text you have read, click-to-jump and cue navigation.
- Global hotkeys: `Ctrl+Alt+Space` start or pause, `Ctrl+Alt+Up/Down` speed, `Ctrl+Alt+Left/Right` cues, `Ctrl+Alt+P` show or hide.
- A script aligner that tracks spoken words against the script. It is tested but not yet connected to a microphone.

## Development

You need Node 22+, Rust (stable) and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```sh
npm install
npm run tauri dev
```

Tests and checks:

```sh
cargo test --workspace
npm run check
```

## Layout

```
crates/eyeline-core   script parser, word aligner, settings, storage, license token check
src-tauri             Tauri shell: windows, commands, hotkeys
src                   Svelte UI for the library window and the prompter
```

## License

MIT
