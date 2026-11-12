<script lang="ts">
  import * as api from '../../api';
  import { confirm } from '../../state/modal.svelte';
  import { busy, runOp } from '../../state/ops';
  import { repo } from '../../state/repo.svelte';
  import { splitStatus } from '../../util/status';
  import { bannerText, opName } from '../../util/wip';

  const op = $derived(repo.state.operation);
  const conflicted = $derived(splitStatus(repo.state.files).conflicted.length);
  const noun = $derived(op ? opName(op.kind).toLowerCase() : '');
  const working = $derived(busy.size > 0);

  const proceed = () => runOp(`Continue ${noun}`, () => api.continueOp(repo.path!));
  const skip = () => runOp('Skip rebase step', () => api.skipOp(repo.path!));
  async function abort() {
    const ok = await confirm({
      title: `Abort ${noun}`,
      body: `Abort the ${noun} and return to the state before it started? Resolutions made so far are discarded.`,
      danger: true,
      confirmLabel: `Abort ${noun}`,
    });
    if (ok) await runOp(`Abort ${noun}`, () => api.abortOp(repo.path!));
  }
</script>

{#if op}
  <div class="banner" class:clean={conflicted === 0} role="status">
    <span class="text">{bannerText(op, conflicted)}</span>
    <span class="btns">
      <button type="button" disabled={conflicted > 0 || working} title={conflicted > 0 ? 'Resolve all conflicts first' : ''} onclick={proceed}>
        Continue
      </button>
      {#if op.kind === 'rebase'}
        <button type="button" disabled={working} onclick={skip}>Skip</button>
      {/if}
      <button class="danger" type="button" disabled={working} onclick={abort}>Abort</button>
    </span>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    flex: none;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--line);
    background: color-mix(in srgb, var(--del) 22%, var(--panel));
    font-size: 12px;
  }
  .banner.clean {
    background: color-mix(in srgb, var(--add) 20%, var(--panel));
  }
  .btns {
    display: inline-flex;
    gap: 6px;
  }
  .danger {
    color: var(--del);
  }
</style>
