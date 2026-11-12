<script lang="ts">
  import * as api from '../../api';
  import { repo } from '../../state/repo.svelte';
  import type { ChangedFile, CommitDetails, Person } from '../../types';
  import { changeSummary } from '../../util/changes';
  import { formatDate } from '../../util/date';
  import { palette } from '../../util/graphGeometry';
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
  const colorOf = (email: string) => {
    let h = 0;
    for (const c of email) h = (h * 31 + c.charCodeAt(0)) >>> 0;
    return palette[h % palette.length];
  };
  const committerDiffers = $derived(
    !!details &&
      (details.author.name !== details.committer.name ||
        details.author.email !== details.committer.email ||
        details.author.time !== details.committer.time),
  );
  const selectedPath = $derived(repo.centre.kind === 'diff' ? repo.centre.path : null);

  function open(file: ChangedFile) {
    repo.centre = { kind: 'diff', path: file.path, origPath: file.origPath, source: { kind: 'commit', oid }, view: 'diff' };
  }
</script>

{#snippet person(label: string, p: Person)}
  <div class="person">
    <Avatar email={p.email} name={p.name} size={24} color={colorOf(p.email)} />
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
        <span class="muted">commit:</span> <span class="mono">{short(details.oid)}</span>
        {#each details.parents as p (p)}
          <span class="muted">parent:</span>
          <button class="link mono" type="button" title={p} onclick={() => repo.revealCommit(p)}>{short(p)}</button>
        {/each}
      </div>
      <h2 class="summary">{details.summary}</h2>
      {#if details.body.trim()}<div class="body">{details.body}</div>{/if}
      <div class="people">
        {@render person('authored', details.author)}
        {#if committerDiffers}{@render person('committed', details.committer)}{/if}
        {#if details.coAuthors.length}
          <div class="co">
            <span class="muted small">Co-authors</span>
            {#each details.coAuthors as c (c.email + c.name)}
              <Avatar email={c.email} name={c.name} size={20} color={colorOf(c.email)} />
            {/each}
          </div>
        {/if}
      </div>
      <div class="muted small count">{changeSummary(details.files) || 'No changed files'}</div>
    </div>
    <div class="files">
      <FileList files={details.files} onOpen={open} selected={selectedPath} />
    </div>
  {/if}
</div>

<style>
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
    font-size: 12px;
  }
  .err {
    color: var(--del);
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
    font-size: 16px;
    font-weight: 600;
    line-height: 1.3;
    overflow-wrap: anywhere;
  }
  .body {
    max-height: 40cqh;
    overflow-y: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-size: 12px;
    color: var(--muted);
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
    font-size: 12px;
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
