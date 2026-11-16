import { createPersister } from './persist';

interface SettingsData { fetchIntervalMin: number; avatarsEnabled: boolean; editorCommand: string; autoUpdate: boolean }

const persister = createPersister<SettingsData>('settings.json');

class Settings implements SettingsData {
  /** Background fetch interval in minutes; 0 = off. */
  fetchIntervalMin = $state(5);
  avatarsEnabled = $state(true);
  editorCommand = $state('');
  /** Check GitHub for new versions in the background. */
  autoUpdate = $state(true);
  /** True once the saved settings have been read (defaults stand in until then). */
  loaded = $state(false);

  async init() {
    const d = await persister.read();
    if (typeof d.fetchIntervalMin === 'number') this.fetchIntervalMin = d.fetchIntervalMin;
    if (typeof d.avatarsEnabled === 'boolean') this.avatarsEnabled = d.avatarsEnabled;
    if (typeof d.editorCommand === 'string') this.editorCommand = d.editorCommand;
    if (typeof d.autoUpdate === 'boolean') this.autoUpdate = d.autoUpdate;
    this.loaded = true;
    $effect.root(() => {
      $effect(() => {
        persister.write({
          fetchIntervalMin: this.fetchIntervalMin,
          avatarsEnabled: this.avatarsEnabled,
          editorCommand: this.editorCommand,
          autoUpdate: this.autoUpdate,
        });
      });
    });
  }
}

export const settings = new Settings();
