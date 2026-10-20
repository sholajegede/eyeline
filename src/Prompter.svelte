<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { api } from "./lib/api";
  import type { ParsedScript, PrompterAction, Settings, Word } from "./lib/types";

  interface Paragraph {
    cueTitle: string | null;
    words: { word: Word; index: number }[];
  }

  const SPEED_STEP = 10;
  const MIN_SPEED = 4;
  const MAX_SPEED = 400;

  let settings = $state<Settings | null>(null);
  let script = $state<ParsedScript>({ words: [], cues: [] });
  let loaded = $state(false);
  let playing = $state(false);
  let ended = $state(false);
  let paused = $state(false);
  let countdown = $state<number | null>(null);
  let speed = $state(60);
  let toast = $state("");
  let padTop = $state(0);

  let viewport: HTMLDivElement | undefined = $state();
  let content: HTMLDivElement | undefined = $state();

  let offset = 0;
  let maxOffset = 0;
  let tops: number[] = [];
  let cueTops: number[] = [];
  let spans: HTMLElement[] = [];
  let readMark = 0;
  let frameId = 0;
  let lastStamp = 0;
  let countdownTimer: number | undefined;
  let toastTimer: number | undefined;

  const paragraphs = $derived.by(() => {
    const showCues = script.cues.length > 1;
    const titles = new Map(script.cues.map((c) => [c.start, c.title]));
    const out: Paragraph[] = [];
    script.words.forEach((word, index) => {
      const title = showCues ? (titles.get(index) ?? null) : null;
      if (word.paragraph_start || title !== null || out.length === 0) {
        out.push({ cueTitle: title, words: [] });
      }
      out[out.length - 1].words.push({ word, index });
    });
    return out;
  });

  const mirror = $derived(
    `scale(${settings?.mirror_x ? -1 : 1}, ${settings?.mirror_y ? -1 : 1})`,
  );

  $effect(() => {
    if (!playing) return;
    lastStamp = 0;
    frameId = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frameId);
  });

  function step(stamp: number) {
    frameId = requestAnimationFrame(step);
    const dt = lastStamp ? Math.min(0.1, (stamp - lastStamp) / 1000) : 0;
    lastStamp = stamp;
    if (paused || countdown !== null) return;
    offset = Math.min(maxOffset, offset + speed * dt);
    apply();
    if (offset >= maxOffset) {
      playing = false;
      ended = true;
    }
  }

  function measure() {
    if (!viewport || !content) return;
    padTop = Math.round(viewport.clientHeight * 0.3);
    spans = Array.from(content.querySelectorAll<HTMLElement>(".w"));
    tops = spans.map((s) => s.offsetTop);
    cueTops = script.cues.map((c) => tops[c.start] ?? 0);
    maxOffset = Math.max(0, (tops.at(-1) ?? 0) - padTop);
    offset = Math.min(offset, maxOffset);
  }

  function apply() {
    if (!content) return;
    content.style.transform = `translate3d(0, ${-offset}px, 0)`;
    updateRead();
  }

  function updateRead() {
    const limit = offset + padTop - 2;
    let mark = readMark;
    while (mark < tops.length && tops[mark] < limit) mark++;
    while (mark > 0 && tops[mark - 1] >= limit) mark--;
    for (let i = readMark; i < mark; i++) spans[i]?.classList.add("read");
    for (let i = mark; i < readMark; i++) spans[i]?.classList.remove("read");
    readMark = mark;
  }

  function seek(next: number) {
    offset = Math.max(0, Math.min(maxOffset, next));
    ended = ended && offset >= maxOffset;
    apply();
  }

  function jumpToWord(index: number) {
    const top = tops[index];
    if (top !== undefined) seek(top - padTop);
  }

  function nextCue() {
    const target = cueTops.find((top) => top - padTop > offset + 8);
    if (target !== undefined) seek(target - padTop);
  }

  function previousCue() {
    const earlier = cueTops.filter((top) => top - padTop < offset - 8);
    seek(earlier.length > 0 ? earlier[earlier.length - 1] - padTop : 0);
  }

  function say(message: string) {
    toast = message;
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => (toast = ""), 900);
  }

  function setSpeed(next: number) {
    speed = Math.max(MIN_SPEED, Math.min(MAX_SPEED, next));
    say(`${speed} px/s`);
  }

  function cancelCountdown() {
    window.clearInterval(countdownTimer);
    countdown = null;
  }

  function toggle() {
    if (countdown !== null) {
      cancelCountdown();
      return;
    }
    if (playing) {
      playing = false;
      return;
    }
    if (ended) {
      ended = false;
      seek(0);
    }
    const seconds = settings?.countdown_secs ?? 0;
    if (offset < 1 && seconds > 0) {
      countdown = seconds;
      countdownTimer = window.setInterval(() => {
        countdown = (countdown ?? 1) - 1;
        if (countdown <= 0) {
          cancelCountdown();
          playing = true;
        }
      }, 1000);
    } else {
      playing = true;
    }
  }

  function act(action: PrompterAction) {
    switch (action) {
      case "toggle":
        return toggle();
      case "faster":
        return setSpeed(speed + SPEED_STEP);
      case "slower":
        return setSpeed(speed - SPEED_STEP);
      case "next-cue":
        return nextCue();
      case "previous-cue":
        return previousCue();
    }
  }

  async function load(id: string) {
    const text = await api.readScript(id);
    script = await api.parseScript(text);
    cancelCountdown();
    playing = false;
    ended = false;
    offset = 0;
    readMark = 0;
    loaded = true;
    await tick();
    measure();
    apply();
  }

  function onKey(event: KeyboardEvent) {
    const keys: Record<string, PrompterAction> = {
      " ": "toggle",
      ArrowUp: "faster",
      ArrowDown: "slower",
      ArrowRight: "next-cue",
      ArrowLeft: "previous-cue",
    };
    if (event.key === "Escape") {
      void api.closePrompter();
    } else if (keys[event.key]) {
      event.preventDefault();
      act(keys[event.key]);
    }
  }

  function onWheel(event: WheelEvent) {
    seek(offset + event.deltaY);
  }

  function onContentClick(event: MouseEvent) {
    const target = (event.target as HTMLElement).closest<HTMLElement>(".w");
    if (target?.dataset.i) jumpToWord(Number(target.dataset.i));
  }

  onMount(() => {
    const unlisten = [
      listen<string>("prompter-load", (e) => load(e.payload)),
      listen<PrompterAction>("prompter-action", (e) => act(e.payload)),
      listen<Settings>("settings-changed", async (e) => {
        settings = e.payload;
        speed = e.payload.speed;
        await tick();
        measure();
        apply();
      }),
    ];

    const resize = new ResizeObserver(() => {
      measure();
      apply();
    });
    if (viewport) resize.observe(viewport);

    (async () => {
      settings = await api.getSettings();
      speed = settings.speed;
      const id = await api.getActiveScript();
      if (id) await load(id);
      void document.fonts.ready.then(() => {
        measure();
        apply();
      });
    })();

    return () => {
      resize.disconnect();
      cancelCountdown();
      window.clearTimeout(toastTimer);
      unlisten.forEach((p) => p.then((fn) => fn()));
    };
  });
</script>

<svelte:window onkeydown={onKey} />

<div
  class="stage"
  class:notch={settings?.mode !== "floating"}
  class:floating={settings?.mode === "floating"}
  class:dim={settings?.dim_read}
  style:--fs="{settings?.font_size ?? 30}px"
  style:--fg={settings?.text_color ?? "#f5f5f0"}
  style:--pad-top="{padTop}px"
  onpointerenter={() => (paused = true)}
  onpointerleave={() => (paused = false)}
  role="application"
  aria-label="Teleprompter"
>
  <div class="drag" data-tauri-drag-region></div>

  <div class="mirror" style:transform={mirror}>
    <div class="viewport" bind:this={viewport} onwheel={onWheel}>
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="content" bind:this={content} onclick={onContentClick}>
        {#each paragraphs as para}
          <p class="para">
            {#if para.cueTitle}<span class="cue-tag">{para.cueTitle}</span>{/if}
            {#each para.words as { word, index }}
              <span class="w" class:direction={!word.spoken} data-i={index}>{word.text}</span>{" "}
            {/each}
          </p>
        {/each}
        {#if loaded && script.words.length === 0}
          <p class="para empty">This script is empty.</p>
        {/if}
      </div>
    </div>

    <div class="marker" aria-hidden="true"></div>

    {#if countdown !== null}
      <div class="hud countdown" aria-live="assertive">{countdown}</div>
    {:else if ended}
      <div class="hud end">End of script</div>
    {:else if paused && playing}
      <div class="hud pill">Paused</div>
    {:else if toast}
      <div class="hud pill" role="status">{toast}</div>
    {/if}
  </div>

  <div class="controls">
    <button type="button" onclick={previousCue} aria-label="Previous cue">⏮</button>
    <button type="button" onclick={toggle} aria-label={playing ? "Pause" : "Start"}>
      {playing || countdown !== null ? "⏸" : "▶"}
    </button>
    <button type="button" onclick={nextCue} aria-label="Next cue">⏭</button>
    <button type="button" onclick={() => setSpeed(speed - SPEED_STEP)} aria-label="Slower">−</button>
    <button type="button" onclick={() => setSpeed(speed + SPEED_STEP)} aria-label="Faster">+</button>
    <button type="button" onclick={() => api.closePrompter()} aria-label="Close prompter">✕</button>
  </div>
</div>

<style>
  .stage {
    position: fixed;
    inset: 0;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }

  .stage.notch {
    background: #000;
    border-radius: 0 0 26px 26px;
  }

  .stage.floating {
    background: rgba(8, 9, 12, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.09);
    border-radius: 16px;
  }

  .drag {
    position: absolute;
    inset: 0 0 auto 0;
    height: 14px;
    z-index: 3;
  }

  .mirror {
    position: absolute;
    inset: 0;
  }

  .viewport {
    position: absolute;
    inset: 0;
    overflow: hidden;
    mask-image: linear-gradient(to bottom, transparent 0, #000 16%, #000 84%, transparent 100%);
  }

  .content {
    padding: var(--pad-top) 7% 70vh;
    text-align: center;
    font-size: var(--fs);
    font-weight: 600;
    line-height: 1.38;
    color: var(--fg);
    will-change: transform;
    cursor: default;
  }

  .para {
    margin: 0 0 0.7em;
  }

  .cue-tag {
    display: block;
    margin-bottom: 0.4em;
    font-size: 0.4em;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: var(--accent);
    opacity: 0.85;
  }

  .w {
    transition: opacity 0.25s ease;
  }

  .w.direction {
    color: var(--accent);
    font-style: italic;
    font-weight: 500;
    opacity: 0.75;
  }

  .dim :global(.w.read) {
    opacity: 0.28;
  }

  .empty {
    opacity: 0.5;
  }

  .marker {
    position: absolute;
    left: 6px;
    top: calc(var(--pad-top) + 0.5em);
    width: 0;
    height: 0;
    border-top: 6px solid transparent;
    border-bottom: 6px solid transparent;
    border-left: 8px solid var(--accent);
    opacity: 0.8;
    pointer-events: none;
  }

  .hud {
    position: absolute;
    pointer-events: none;
  }

  .countdown {
    inset: 0;
    display: grid;
    place-items: center;
    font-size: min(70vh, 160px);
    font-weight: 700;
    color: var(--fg);
    background: rgba(0, 0, 0, 0.55);
  }

  .end {
    inset: auto 0 14px 0;
    text-align: center;
    color: var(--accent);
    font-size: 13px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .pill {
    right: 14px;
    bottom: 12px;
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.14);
    color: #fff;
    font-size: 12px;
    font-weight: 500;
  }

  .controls {
    position: absolute;
    top: 8px;
    right: 10px;
    z-index: 4;
    display: flex;
    gap: 4px;
    padding: 3px;
    border-radius: 10px;
    background: rgba(30, 32, 38, 0.92);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .stage:hover .controls,
  .controls:focus-within {
    opacity: 1;
  }

  .controls button {
    width: 28px;
    height: 26px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: #eee;
    font-size: 13px;
    line-height: 1;
  }

  .controls button:hover {
    background: rgba(255, 255, 255, 0.12);
  }
</style>
