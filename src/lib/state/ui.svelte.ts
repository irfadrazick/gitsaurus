/** Cross-component UI flags (Ruling R3). The shortcut handler sets these; SearchBar and Settings bind to them. */
/** `searchFocus` is bumped to ask an already-open SearchBar to refocus its field. */
export const ui = $state({ searchOpen: false, settingsOpen: false, searchFocus: 0 });
