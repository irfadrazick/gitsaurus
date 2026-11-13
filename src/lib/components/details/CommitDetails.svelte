<script lang="ts">
  import * as api from '../../api';
  import { copyText } from '../../state/actions';
  import { repo } from '../../state/repo.svelte';
  import type { ChangedFile, CommitDetails, Person } from '../../types';
  import { changeSummary } from '../../util/changes';
  import { formatDate } from '../../util/date';
  import { displayBody } from '../../util/message';
  import { personColor } from '../../util/avatar';
  import Avatar from '../graph/Avatar.svelte';
  import FileList from './FileList.svelte';

  let { oid }: { oid: string } = $props();

  let details = $state<CommitDetails | null>(null);
  let error = $state<string | null>(null);
  let token = 0;

  $effect(() => {
    const path = repo.path;
    const id = oid;
    const t = ++token;
    details = null;
    error = null;
    if (!path) return;
    api.commitDetails(path, id).then(
      (d) => {
        if (t === token) details = d;
      },
      (e) => {
        if (t === token) error = (e as { message?: string })?.message ?? String(e);
      },
    );
  });

  const short = (s: string) => s.slice(0, 7);
  const committerDiffers = $derived(
    !!details &&
      (details.author.name !== details.committer.name ||
        details.author.email !== details.committer.email ||
        details.author.time !== details.committer.time),
  );
  const body = $derived(details ? displayBody(details.body) : '');
  const selectedPath = $derived(repo.centre.kind === 'diff' ? repo.centre.path : null);
  const outsideGraph = $derived(repo.selection?.kind === 'commit' && repo.selection.oid === oid && !!repo.selection.outsideGraph);

  function open(file: ChangedFile) {
    repo.centre = { kind: 'diff', path: file.path, origPath: file.origPath, source: { kind: 'commit', oid }, view: 'diff' };
  }
</script>

{#snippet person(label: string, p: Person)}
  <div class="person">
    <Avatar email={p.email} name={p.name} size={24} color={personColor(p.email)} />
    <div class="who">
      <span class="name" title={p.email}>{p.name}</span>
      <span class="muted small">{label} {formatDate(p.time)}</span>
    </div>
  </div>
{/snippet}

<div class="details">
  {#if error}
    <div class="msg err">{error}</div>
  {:else if !details}
    <div class="msg muted">Loading…</div>
  {:else}
    <div class="head">
      <div class="shas small">
        {#if outsideGraph}<span class="note">Stash (not in graph)</span>{/if}
        <span class="muted">Commit</span>
        <button class="sha mono" type="button" title="Copy {details.oid}" onclick={() => copyText(details!.oid)}>{short(details.oid)}</button>
        {#if details.parents.length}
          <span class="muted">{details.parents.length > 1 ? 'Parents' : 'Parent'}</span>
          {#each details.parents as p (p)}
            <button class="link mono" type="button" title="Show {p}" onclick={() => repo.revealCommit(p)}>{short(p)}</button>
          {/each}
        {/if}
      </div>
      <h2 class="summary">{details.summary}</h2>
      {#if body}<div class="body">{body}</div>{/if}
      <div class="people">
        {@render person('authored', details.author)}
        {#if committerDiffers}{@render person('committed', details.committer)}{/if}
        {#if details.coAuthors.length}
          <div class="co">
            <span class="muted small">Co-authors</span>
            {#each details.coAuthors as c (c.email + c.name)}
              <Avatar email={c.email} name={c.name} size={20} color={personColor(c.email)} />
            {/each}
          </div>
        {/if}
      </div>
    </div>
    <div class="files">
      <FileList files={details.files} onOpen={open} selected={selectedPath} summary={changeSummary(details.files) || 'No changed files'} />
    </div>
  {/if}
</div>

<style>
  .note {
    margin-right: 6px;
    padding: 0 6px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    color: var(--muted);
  }
  .details {
    container-type: size;
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    user-select: text;
    -webkit-user-select: text;
  }
  .msg {
    padding: 10px 12px;
    font-size: var(--fs-sm);
  }
  .err {
    color: var(--danger-text);
  }
  .sha {
    padding: 0 4px;
    margin: 0 -4px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
  }
  .sha:hover:not(:disabled) {
    background: var(--hover);
  }
  .head {
    display: flex;
    flex: none;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    max-height: 70%;
    overflow-y: auto;
    border-bottom: 1px solid var(--line);
  }
  .shas {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 6px;
    align-items: baseline;
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--lane-1);
    text-decoration: underline;
    text-decoration-color: transparent;
  }
  .link:hover {
    text-decoration-color: currentColor;
  }
  .summary {
    margin: 0;
    font-size: var(--fs-lg);
    font-weight: 600;
    line-height: 1.3;
    overflow-wrap: anywhere;
  }
  .body {
    max-height: 40cqh;
    overflow-y: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-size: var(--fs-sm);
    line-height: 1.5;
    color: color-mix(in srgb, var(--text) 85%, var(--muted));
  }
  .people {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .person,
  .co {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .who {
    display: flex;
    min-width: 0;
    flex-direction: column;
    line-height: 1.25;
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--fs-sm);
  }
  .co {
    flex-wrap: wrap;
    gap: 4px;
  }
  .co .small {
    margin-right: 2px;
  }
  .files {
    flex: 1;
    min-height: 0;
  }
</style>
