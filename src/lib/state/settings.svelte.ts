import { createPersister } from './persist';

interface SettingsData { fetchIntervalMin: number; avatarsEnabled: boolean; editorCommand: string }

const persister = createPersister<SettingsData>('settings.json');

class Settings implements SettingsData {
  /** Background fetch interval in minutes; 0 = off. */
  fetchIntervalMin = $state(5);
  avatarsEnabled = $state(true);
  editorCommand = $state('');
  /** True once the saved settings have been read (defaults stand in until then). */
  loaded = $state(false);

  async init() {
    const d = await persister.read();
    if (typeof d.fetchIntervalMin === 'number') this.fetchIntervalMin = d.fetchIntervalMin;
    if (typeof d.avatarsEnabled === 'boolean') this.avatarsEnabled = d.avatarsEnabled;
    if (typeof d.editorCommand === 'string') this.editorCommand = d.editorCommand;
    this.loaded = true;
    $effect.root(() => {
      $effect(() => {
        persister.write({
          fetchIntervalMin: this.fetchIntervalMin,
          avatarsEnabled: this.avatarsEnabled,
          editorCommand: this.editorCommand,
        });
      });
    });
  }
}

export const settings = new Settings();
