<script lang="ts">
  import { avatarFailed, avatarFor } from '../../state/avatars.svelte';
  import { initials } from '../../util/avatar';

  let { email, name, size = 20, color }: { email: string; name: string; size?: number; color: string } = $props();

  const url = $derived(avatarFor(email));
</script>

<span class="av" style:width="{size}px" style:height="{size}px" style:background={color} title={`${name} <${email}>`}>
  <span class="ini">{initials(name)}</span>
  {#if url}
    <img src={url} alt="" draggable="false" onerror={() => avatarFailed(email)} />
  {/if}
</span>

<style>
  .av {
    position: relative;
    display: inline-flex;
    flex: none;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border-radius: 50%;
  }
  .ini {
    color: #11151b;
    font-size: var(--fs-xs);
    font-weight: 700;
    letter-spacing: -0.03em;
    line-height: 1;
  }
  img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
