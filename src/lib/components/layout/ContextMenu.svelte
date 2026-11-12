<script lang="ts">
  import { closeMenu, contextMenu } from '../../state/contextMenu.svelte';
  import type { MenuItem } from '../../util/menus';

  const MENU_W = 240;
  const cur = $derived(contextMenu.current);
  let w = $state(MENU_W);
  let h = $state(0);
  let openSub = $state<string | null>(null);

  const left = $derived(cur ? Math.max(4, Math.min(cur.x, window.innerWidth - w - 4)) : 0);
  const top = $derived(cur ? Math.max(4, Math.min(cur.y, window.innerHeight - h - 4)) : 0);
  const flip = $derived(left + w * 2 + 8 > window.innerWidth);

  $effect(() => {
    if (cur) openSub = null;
  });

  function pick(item: MenuItem) {
    if (item.disabled || item.submenu) return;
    const m = cur;
    closeMenu();
    m?.onPick(item.id);
  }
</script>

<svelte:window
  onkeydowncapture={cur ? (e) => e.key === 'Escape' && (e.preventDefault(), e.stopPropagation(), closeMenu()) : undefined}
  onblur={closeMenu}
  onresize={closeMenu}
/>

{#if cur}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="scrim" onpointerdown={closeMenu} oncontextmenu={(e) => { e.preventDefault(); closeMenu(); }}></div>
  <div class="menu" role="menu" style:left="{left}px" style:top="{top}px" bind:clientWidth={w} bind:clientHeight={h}>
    {#each cur.items as item (item.id)}
      {#if item.separatorBefore}<div class="sep" role="separator"></div>{/if}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="wrap" onmouseenter={() => (openSub = item.submenu ? item.id : null)}>
        <button
          class="item"
          class:danger={item.danger}
          class:has-sub={!!item.submenu}
          role="menuitem"
          disabled={item.disabled}
          aria-haspopup={item.submenu ? 'menu' : undefined}
          onclick={() => pick(item)}
        >
          <span class="label">{item.label}</span>
          {#if item.submenu}<span class="chev" aria-hidden="true">›</span>{/if}
        </button>
        {#if item.submenu && openSub === item.id}
          <div class="menu sub" class:flip role="menu">
            {#each item.submenu as sub (sub.id)}
              <button class="item" class:danger={sub.danger} role="menuitem" disabled={sub.disabled} onclick={() => pick(sub)}>
                <span class="label">{sub.label}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 900;
  }
  .menu {
    position: fixed;
    z-index: 901;
    min-width: 200px;
    max-width: 320px;
    padding: 4px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 8px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.45), 0 2px 6px rgba(0, 0, 0, 0.3);
  }
  .menu.sub {
    position: absolute;
    top: -5px;
    left: calc(100% + 2px);
  }
  .menu.sub.flip {
    left: auto;
    right: calc(100% + 2px);
  }
  .wrap {
    position: relative;
  }
  .sep {
    height: 1px;
    margin: 4px 6px;
    background: var(--line);
  }
  .item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    width: 100%;
    height: 28px;
    padding: 0 10px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--text);
    text-align: left;
    font-size: 13px;
  }
  .item:hover:not(:disabled),
  .item:focus-visible {
    background: var(--accent);
    color: #fff;
    border: 0;
    outline: none;
  }
  .item:disabled {
    color: var(--muted);
    opacity: 0.55;
    cursor: default;
  }
  .item.danger {
    color: var(--del);
  }
  .item.danger:hover:not(:disabled) {
    background: var(--del);
    color: #fff;
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chev {
    color: var(--muted);
  }
  .item:hover .chev {
    color: inherit;
  }
</style>
