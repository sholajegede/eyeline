<script lang="ts">
  import type { Settings } from "./lib/types";

  interface Props {
    settings: Settings;
    onchange: () => void;
  }

  let { settings = $bindable(), onchange }: Props = $props();
</script>

<section class="panel" aria-label="Prompter settings">
  <fieldset>
    <legend>Position</legend>
    <div class="segmented" role="radiogroup" aria-label="Prompter position">
      <label class:on={settings.mode === "notch"}>
        <input type="radio" value="notch" bind:group={settings.mode} onchange={onchange} />
        Under camera
      </label>
      <label class:on={settings.mode === "floating"}>
        <input type="radio" value="floating" bind:group={settings.mode} onchange={onchange} />
        Floating
      </label>
    </div>
    <p class="hint">Applies the next time you start the prompter.</p>
  </fieldset>

  <fieldset>
    <legend>Text</legend>
    <label class="field">
      <span>Size <output>{settings.font_size}px</output></span>
      <input type="range" min="14" max="96" step="1" bind:value={settings.font_size} oninput={onchange} />
    </label>
    <label class="field">
      <span>Scroll speed <output>{settings.speed} px/s</output></span>
      <input type="range" min="4" max="400" step="2" bind:value={settings.speed} oninput={onchange} />
    </label>
    <label class="field inline">
      <span>Color</span>
      <input type="color" bind:value={settings.text_color} oninput={onchange} />
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={settings.dim_read} onchange={onchange} />
      Dim text you already read
    </label>
  </fieldset>

  <fieldset>
    <legend>Window</legend>
    <div class="pair">
      <label class="field">
        <span>Width</span>
        <input type="number" min="280" max="2400" step="10" bind:value={settings.width} onchange={onchange} />
      </label>
      <label class="field">
        <span>Height</span>
        <input type="number" min="80" max="1400" step="10" bind:value={settings.height} onchange={onchange} />
      </label>
    </div>
    <label class="field">
      <span>Countdown <output>{settings.countdown_secs}s</output></span>
      <input type="range" min="0" max="10" step="1" bind:value={settings.countdown_secs} oninput={onchange} />
    </label>
  </fieldset>

  <fieldset>
    <legend>Display</legend>
    <label class="check">
      <input type="checkbox" bind:checked={settings.mirror_x} onchange={onchange} />
      Mirror horizontally
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={settings.mirror_y} onchange={onchange} />
      Flip vertically
    </label>
    <label class="check">
      <input type="checkbox" bind:checked={settings.hide_from_capture} onchange={onchange} />
      Hide from screen sharing and recordings
    </label>
  </fieldset>

  <fieldset>
    <legend>Hotkeys</legend>
    <dl>
      <dt>Start / pause</dt><dd>Ctrl + Alt + Space</dd>
      <dt>Faster / slower</dt><dd>Ctrl + Alt + ↑ / ↓</dd>
      <dt>Next / previous cue</dt><dd>Ctrl + Alt + → / ←</dd>
      <dt>Show / hide</dt><dd>Ctrl + Alt + P</dd>
    </dl>
  </fieldset>
</section>

<style>
  .panel {
    width: 292px;
    flex: none;
    overflow-y: auto;
    background: var(--surface);
    border-left: 1px solid var(--line);
    padding: 14px 16px 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  fieldset {
    border: 0;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  legend {
    padding: 0;
    margin-bottom: 6px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field.inline {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }

  .field span {
    display: flex;
    justify-content: space-between;
  }

  output {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  input[type="range"] {
    width: 100%;
    accent-color: var(--accent);
  }

  input[type="number"] {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 6px 8px;
  }

  input[type="color"] {
    width: 44px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--line);
    border-radius: 6px;
    background: none;
  }

  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .check {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .check input {
    accent-color: var(--accent);
  }

  .segmented {
    display: grid;
    grid-template-columns: 1fr 1fr;
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: 9px;
    padding: 3px;
  }

  .segmented label {
    text-align: center;
    padding: 6px 4px;
    border-radius: 6px;
    cursor: pointer;
  }

  .segmented label.on {
    background: var(--surface-2);
    font-weight: 600;
  }

  .segmented input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .segmented label:has(input:focus-visible) {
    outline: 2px solid var(--accent);
  }

  .hint {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 12px;
    margin: 0;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    text-align: right;
    font-family: var(--mono);
    font-size: 12px;
  }
</style>
