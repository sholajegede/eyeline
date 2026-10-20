<script lang="ts">
  import { relativeTime } from "./lib/format";
  import type { ScriptMeta } from "./lib/types";

  interface Props {
    scripts: ScriptMeta[];
    activeId: string | null;
    onselect: (id: string) => void;
    oncreate: () => void;
    ondelete: (id: string) => void;
  }

  let { scripts, activeId, onselect, oncreate, ondelete }: Props = $props();

  let query = $state("");
  let confirming = $state<string | null>(null);

  const visible = $derived(
    scripts.filter((s) => s.title.toLowerCase().includes(query.trim().toLowerCase())),
  );

  function askDelete(id: string) {
    if (confirming === id) {
      confirming = null;
      ondelete(id);
    } else {
      confirming = id;
    }
  }
</script>

<aside class="library" aria-label="Scripts">
  <header data-tauri-drag-region>
    <span class="brand" data-tauri-drag-region>Eyeline</span>
    <button class="new" type="button" onclick={oncreate} aria-label="New script">New</button>
  </header>

  <div class="search">
    <input
      type="search"
      placeholder="Search scripts"
      bind:value={query}
      aria-label="Search scripts"
    />
  </div>

  <ul>
    {#each visible as script (script.id)}
      <li class:active={script.id === activeId}>
        <button class="row" type="button" onclick={() => onselect(script.id)}>
          <span class="title">{script.title}</span>
          <span class="when">{relativeTime(script.modified)}</span>
        </button>
        <button
          class="delete"
          class:armed={confirming === script.id}
          type="button"
          onclick={() => askDelete(script.id)}
          onblur={() => (confirming = null)}
          aria-label={confirming === script.id ? `Confirm delete ${script.title}` : `Delete ${script.title}`}
        >
          {confirming === script.id ? "Delete?" : "×"}
        </button>
      </li>
    {:else}
      <li class="empty">{query ? "No scripts match." : "No scripts yet."}</li>
    {/each}
  </ul>
</aside>

<style>
  .library {
    display: flex;
    flex-direction: column;
    width: 264px;
    flex: none;
    background: var(--surface);
    border-right: 1px solid var(--line);
    min-height: 0;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 14px 10px 16px;
  }

  :global(html.mac) header {
    padding-top: 40px;
  }

  .brand {
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  .new {
    background: var(--accent);
    color: var(--accent-ink);
    border: 0;
    border-radius: 8px;
    padding: 5px 12px;
    font-weight: 600;
  }

  .new:hover {
    filter: brightness(1.08);
  }

  .search {
    padding: 0 12px 10px;
  }

  input[type="search"] {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 7px 10px;
    user-select: text;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0 8px 12px;
    overflow-y: auto;
    flex: 1;
  }

  li {
    display: flex;
    align-items: stretch;
    border-radius: 8px;
  }

  li.active {
    background: var(--surface-2);
  }

  li:hover:not(.active):not(.empty) {
    background: color-mix(in srgb, var(--surface-2) 55%, transparent);
  }

  .row {
    flex: 1;
    min-width: 0;
    text-align: left;
    background: none;
    border: 0;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .when {
    color: var(--muted);
    font-size: 12px;
  }

  .delete {
    background: none;
    border: 0;
    color: var(--muted);
    padding: 0 10px;
    border-radius: 8px;
    opacity: 0;
  }

  li:hover .delete,
  .delete:focus-visible,
  .delete.armed {
    opacity: 1;
  }

  .delete.armed {
    color: var(--danger);
    font-weight: 600;
  }

  .empty {
    color: var(--muted);
    padding: 12px 10px;
  }
</style>
