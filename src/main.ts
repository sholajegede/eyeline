import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./app.css";
import App from "./App.svelte";
import Prompter from "./Prompter.svelte";

const isPrompter = getCurrentWindow().label === "prompter";
document.documentElement.dataset.view = isPrompter ? "prompter" : "app";
if (navigator.userAgent.includes("Mac")) {
  document.documentElement.classList.add("mac");
}

mount(isPrompter ? Prompter : App, {
  target: document.getElementById("app")!,
});
