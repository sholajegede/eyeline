<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import { readingMinutes, spokenWordCount } from "./lib/format";
  import type { ScriptMeta, Settings } from "./lib/types";
  import Library from "./Library.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";

  let scripts = $state<ScriptMeta[]>([]);
  let activeId = $state<string | null>(null);
  let text = $state("");
  let settings = $state<Settings | null>(null);
  let showSettings = $state(true);
  let status = $state<"idle" | "saving" | "saved" | "error">("idle");
  let message = $state("");

  let saveTimer: number | undefined;
  let settingsTimer: number | undefined;
  let loadToken = 0;

  const words = $derived(spokenWordCount(text));
  const cueCount = $derived((text.match(/^\s*#{1,3}\s/gm) ?? []).length || (text.trim() ? 1 : 0));
  const title = $derived(scripts.find((s) => s.id === activeId)?.title ?? "");

  onMount(async () => {
    try {
      settings = await api.getSettings();
      await refresh();
      if (scripts.length > 0) await select(scripts[0].id);
    } catch (err) {
      fail(err);
    }
  });

  function fail(err: unknown) {
    status = "error";
    message = String(err);
  }

  async function refresh() {
    scripts = await api.listScripts();
  }

  async function select(id: string) {
    await flush();
    const token = ++loadToken;
    try {
      const content = await api.readScript(id);
      if (token !== loadToken) return;
      activeId = id;
      text = content;
      status = "idle";
    } catch (err) {
      fail(err);
    }
  }

  function onInput(event: Event) {
    text = (event.currentTarget as HTMLTextAreaElement).value;
    status = "saving";
    window.clearTimeout(saveTimer);
    saveTimer = window.setTimeout(flush, 450);
  }

  async function flush() {
    window.clearTimeout(saveTimer);
    if (status !== "saving" || !activeId) return;
    try {
      await api.writeScript(activeId, text);
      status = "saved";
      await refresh();
    } catch (err) {
      fail(err);
    }
  }

  async function create() {
    await flush();
    try {
      const id = await api.createScript("Untitled");
      await refresh();
      await select(id);
      document.getElementById("editor")?.focus();
    } catch (err) {
      fail(err);
    }
  }

  async function remove(id: string) {
    try {
      await api.deleteScript(id);
      await refresh();
      if (id === activeId) {
        activeId = null;
        text = "";
        if (scripts.length > 0) await select(scripts[0].id);
      }
    } catch (err) {
      fail(err);
    }
  }

  async function start() {
    await flush();
    if (!activeId) return;
    try {
      await api.openPrompter(activeId);
    } catch (err) {
      fail(err);
    }
  }

  function queueSettingsSave() {
    window.clearTimeout(settingsTimer);
    settingsTimer = window.setTimeout(async () => {
      if (!settings) return;
      try {
        const clean = await api.saveSettings($state.snapshot(settings));
        if (JSON.stringify(clean) !== JSON.stringify($state.snapshot(settings))) {
          settings = clean;
        }
      } catch (err) {
        fail(err);
      }
    }, 250);
  }
</script>

<div class="shell">
  <Library {scripts} {activeId} onselect={select} oncreate={create} ondelete={remove} />

  <main>
    <header data-tauri-drag-region>
      <h1 data-tauri-drag-region>{title || "No script selected"}</h1>
      <div class="actions">
        <button class="ghost" type="button" onclick={() => (showSettings = !showSettings)} aria-pressed={showSettings}>
          Settings
        </button>
        <button class="primary" type="button" onclick={start} disabled={!activeId}>
          Start prompter
        </button>
      </div>
    </header>

    {#if activeId}
      <textarea
        id="editor"
        value={text}
        oninput={onInput}
        spellcheck="true"
        placeholder="Write or paste your script. Use ## for cues and [brackets] for stage directions."
        aria-label="Script"
      ></textarea>
    {:else}
      <div class="blank">
        <p>Create a script to get started.</p>
        <button class="primary" type="button" onclick={create}>New script</button>
      </div>
    {/if}

    <footer>
      <span>{words} words</span>
      <span>{readingMinutes(words)} at 150 wpm</span>
      <span>{cueCount} {cueCount === 1 ? "cue" : "cues"}</span>
      <span class="state" class:error={status === "error"} role="status">
        {#if status === "saving"}Saving…{:else if status === "saved"}Saved{:else if status === "error"}{message}{/if}
      </span>
    </footer>
  </main>

  {#if showSettings && settings}
    <SettingsPanel bind:settings onchange={queueSettingsSave} />
  {/if}
</div>

<style>
  .shell {
    display: flex;
    height: 100%;
  }

  main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 18px 10px;
  }

  :global(html.mac) main header {
    padding-top: 40px;
  }

  h1 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .actions {
    display: flex;
    gap: 8px;
    flex: none;
  }

  button {
    border-radius: 8px;
    padding: 6px 14px;
    border: 1px solid var(--line);
    background: var(--surface-2);
  }

  button.ghost[aria-pressed="true"] {
    border-color: color-mix(in srgb, var(--accent) 55%, var(--line));
  }

  button.primary {
    background: var(--accent);
    color: var(--accent-ink);
    border-color: transparent;
    font-weight: 600;
  }

  button.primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  textarea {
    flex: 1;
    resize: none;
    border: 0;
    outline: 0;
    background: transparent;
    padding: 12px 28px 20px;
    font: 17px/1.65 var(--font);
    user-select: text;
  }

  textarea:focus-visible {
    outline: none;
  }

  .blank {
    flex: 1;
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 12px;
    color: var(--muted);
  }

  footer {
    display: flex;
    gap: 18px;
    padding: 8px 18px;
    border-top: 1px solid var(--line);
    color: var(--muted);
    font-size: 12px;
  }

  .state {
    margin-left: auto;
  }

  .state.error {
    color: var(--danger);
  }
</style>
