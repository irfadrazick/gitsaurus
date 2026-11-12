<script lang="ts" module>
  /** Small UI icons, drawn on a 16-unit grid in the toolbar's stroke style. */
  const PATHS = {
    chevronDown: 'M4.5 6.5 8 10l3.5-3.5',
    chevronRight: 'M6.5 4.5 10 8l-3.5 3.5',
    check: 'M3.5 8.5 6.5 11.5 12.5 5',
    close: 'M4.5 4.5l7 7M11.5 4.5l-7 7',
    plus: 'M8 3.5v9M3.5 8h9',
    minus: 'M3.5 8h9',
    arrowUp: 'M8 12.5v-9M4.5 7 8 3.5 11.5 7',
    arrowDown: 'M8 3.5v9M4.5 9 8 12.5 11.5 9',
    file: 'M4.5 2.5h4.5l3 3v8h-7.5zM9 2.5v3h3',
    sidebar: 'M2.5 3.5h11v9h-11zM6.5 3.5v9',
    computer: 'M3.5 4h9v6.5h-9zM1.5 12.5h13',
    cloud: 'M4.6 12.5h6.9a2.8 2.8 0 0 0 .3-5.6 4.2 4.2 0 0 0-8-.9 2.9 2.9 0 0 0 .8 6.5z',
    tag: 'M2.5 2.5h5l6 6-5 5-6-6zM5.5 5.5h.01',
    stash: 'M2.5 9 4 3h8l1.5 6M2.5 9v4h11V9M2.5 9h3.5l.5 1.5h3L10 9h3.5',
    home: 'M2.5 7.5 8 3l5.5 4.5M4 6.5v6.5h8V6.5',
    folder: 'M2 4.5h4l1.5 1.5H14v6.5H2z',
    pencil: 'M10.5 3 13 5.5 6 12.5H3.5V10z',
    info: 'M8 7.5v4M8 5h.01M14 8A6 6 0 1 1 2 8a6 6 0 0 1 12 0',
    alert: 'M8 5v3.5M8 11h.01M14 8A6 6 0 1 1 2 8a6 6 0 0 1 12 0',
    checkCircle: 'M5.5 8.2 7.3 10l3.2-3.6M14 8A6 6 0 1 1 2 8a6 6 0 0 1 12 0',
  } as const;
  export type IconName = keyof typeof PATHS;
</script>

<script lang="ts">
  let { name, size = 14, stroke = 1.5 }: { name: IconName; size?: number; stroke?: number } = $props();
</script>

<svg class="icon" width={size} height={size} viewBox="0 0 16 16" aria-hidden="true" stroke-width={stroke}>
  <path d={PATHS[name]} />
</svg>

<style>
  .icon {
    display: block;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
