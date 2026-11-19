<script lang="ts">
  import { tick } from 'svelte';
  import { closeMenu, contextMenu } from '../../state/contextMenu.svelte';
  import type { MenuItem } from '../../util/menus';
  import Icon from '../Icon.svelte';

  const MENU_W = 240;
  const cur = $derived(contextMenu.current);
  let w = $state(MENU_W);
  let h = $state(0);
  let openSub = $state<string | null>(null);
  let menuEl = $state<HTMLDivElement>();

  const left = $derived(cur ? Math.max(4, Math.min(cur.x, window.innerWidth - w - 4)) : 0);
  const top = $derived(cur ? Math.max(4, Math.min(cur.y, window.innerHeight - h - 4)) : 0);
  const flip = $derived(left + w * 2 + 8 > window.innerWidth);

  // Remember what had focus, move focus into the menu, and give it back on close.
  let returnFocus: HTMLElement | null = null;
  $effect(() => {
    if (!cur) return;
    openSub = null;
    returnFocus = document.activeElement as HTMLElement | null;
    tick().then(() => items(menuEl)[0]?.focus());
    return () => returnFocus?.focus?.({ preventScroll: true });
  });

  /** Enabled items of one menu level, in order. */
  const items = (menu: Element | null | undefined) =>
    menu ? [...menu.querySelectorAll<HTMLButtonElement>(':scope > .wrap > .item:not(:disabled), :scope > .item:not(:disabled)')] : [];

  function pick(item: MenuItem) {
    if (item.disabled || item.submenu) return;
    const m = cur;
    // The picked action owns focus from here (it may open a dialog), so don't restore it.
    returnFocus = null;
    closeMenu();
    m?.onPick(item.id);
  }

  async function openSubmenu(item: MenuItem) {
    openSub = item.id;
    await tick();
    const sub = menuEl?.querySelector('.menu.sub');
    items(sub)[0]?.focus();
  }

  function onMenuKey(e: KeyboardEvent) {
    const active = document.activeElement as HTMLElement | null;
    const level = active?.closest('.menu');
    const list = items(level);
    const i = active ? list.indexOf(active as HTMLButtonElement) : -1;
    const move = (to: number) => list[(to + list.length) % list.length]?.focus();
    if (e.key === 'ArrowDown') move(i + 1);
    else if (e.key === 'ArrowUp') move(i < 0 ? list.length - 1 : i - 1);
    else if (e.key === 'Home') move(0);
    else if (e.key === 'End') move(list.length - 1);
    else if (e.key === 'ArrowRight' && active?.classList.contains('has-sub')) {
      const id = active.dataset.id;
      const item = cur?.items.find((it) => it.id === id);
      if (item) void openSubmenu(item);
    } else if (e.key === 'ArrowLeft' && level?.classList.contains('sub')) {
      const owner = level.parentElement?.querySelector<HTMLButtonElement>(':scope > .item');
      openSub = null;
      owner?.focus();
    } else return;
    e.preventDefault();
  }

  // Shift+F10 / the context-menu key opens the menu for whatever has keyboard focus.
  function onWindowKey(e: KeyboardEvent) {
    if (cur) {
      if (e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
        if (openSub) {
          const owner = menuEl?.querySelector<HTMLButtonElement>(`.item[data-id="${openSub}"]`);
          openSub = null;
          owner?.focus();
        } else closeMenu();
      }
      return;
    }
    if (!(e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10'))) return;
    let target = document.activeElement as HTMLElement | null;
    if (!target || target === document.body) return;
    target = target.querySelector<HTMLElement>('.row.selected') ?? target;
    const r = target.getBoundingClientRect();
    e.preventDefault();
    target.dispatchEvent(
      new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: r.left + Math.min(r.width / 2, 160), clientY: r.top + r.height / 2 }),
    );
  }
</script>

<svelte:window onkeydowncapture={onWindowKey} onblur={closeMenu} onresize={closeMenu} />

{#if cur}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="scrim" onpointerdown={closeMenu} oncontextmenu={(e) => { e.preventDefault(); closeMenu(); }}></div>
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="menu" role="menu" tabindex="-1" style:left="{left}px" style:top="{top}px" bind:this={menuEl} bind:clientWidth={w} bind:clientHeight={h} onkeydown={onMenuKey}>
    {#each cur.items as item (item.id)}
      {#if item.separatorBefore}<div class="sep" role="separator"></div>{/if}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="wrap"
        onmouseenter={(e) => {
          openSub = item.submenu ? item.id : null;
          // Hover moves focus too, so mouse and keyboard never highlight two items at once.
          ((e.currentTarget as HTMLElement).querySelector<HTMLElement>(':scope > .item:not(:disabled)') ?? menuEl)?.focus();
        }}
      >
        <button
          class="item"
          class:danger={item.danger}
          class:has-sub={!!item.submenu}
          class:open={openSub === item.id}
          role="menuitem"
          data-id={item.id}
          disabled={item.disabled}
          aria-haspopup={item.submenu ? 'menu' : undefined}
          aria-expanded={item.submenu ? openSub === item.id : undefined}
          onclick={() => (item.submenu ? openSubmenu(item) : pick(item))}
        >
          <span class="label">{item.label}</span>
          {#if item.hint}<span class="hint">{item.hint}</span>{/if}
          {#if item.submenu}<span class="chev" aria-hidden="true"><Icon name="chevronRight" size={12} /></span>{/if}
        </button>
        {#if item.submenu && openSub === item.id}
          <div class="menu sub" class:flip role="menu">
            {#each item.submenu as sub (sub.id)}
              {#if sub.separatorBefore}<div class="sep" role="separator"></div>{/if}
              <button class="item" class:danger={sub.danger} role="menuitem" data-id={sub.id} disabled={sub.disabled} onclick={() => pick(sub)} onmouseenter={(e) => (e.currentTarget as HTMLElement).focus()}>
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
    /* As wide as the longest item (a long branch name), up to the window's width. */
    width: max-content;
    min-width: 200px;
    max-width: calc(100vw - 8px);
    padding: 4px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    box-shadow: 0 10px 30px rgb(0 0 0 / 0.45), 0 2px 6px rgb(0 0 0 / 0.3);
    outline: none;
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
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
    text-align: left;
    font-size: var(--fs-md);
  }
  .item:focus,
  .item.open {
    background: var(--accent);
    color: var(--on-accent);
    border: 0;
    outline: none;
  }
  .item:disabled {
    color: var(--muted);
    opacity: 0.55;
  }
  .item.danger {
    color: var(--danger-text);
  }
  .item.danger:focus {
    background: var(--danger-fill);
    color: var(--on-accent);
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chev {
    display: inline-flex;
    color: var(--muted);
  }
  .hint {
    flex: none;
    color: var(--muted);
    font-size: var(--fs-sm);
  }
  .item:focus .hint {
    color: inherit;
    opacity: 0.8;
  }
  .item:focus .chev,
  .item.open .chev {
    color: inherit;
  }
</style>
