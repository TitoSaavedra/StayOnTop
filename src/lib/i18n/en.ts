const en = {
  nav: {
    windows: 'Windows',
    settings: 'Settings',
  },
  titlebar: {
    minimize: 'Minimize',
    close: 'Close to tray',
  },
  home: {
    pinned: 'Pinned',
    processes: 'Processes',
    unpinAll: 'Unpin all',
    opacity: 'Opacity',
    clickThrough: 'Click-through',
    onTop: 'On Top',
    pin: 'Pin',
    unpin: 'Unpin',
    searchPlaceholder: 'Filter processes…',
  },
  settings: {
    general: 'General',
    language: 'Language',
    startWithWindows: 'Start with Windows',
    keepOnTop: 'Keep app always on top',
    defaultOpacity: 'Default pin opacity',
    processRefresh: 'Process refresh',
    refreshInterval: 'Refresh interval',
    excludedProcesses: 'Excluded processes',
    addPlaceholder: 'explorer.exe',
    add: 'Add',
    save: 'Save settings',
    saving: 'Saving…',
    hotkeys: 'Keyboard shortcuts',
    pinToggle: 'Pin / Unpin selected',
    pressKeys: 'Click and press keys…',
  },
  status: {
    processes: 'processes',
    pinned: 'pinned',
  },
} as const;

export type Translations = typeof en;
export default en;
