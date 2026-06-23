export interface ProcessInfo {
  pid: number;
  name: string;
  window_title: string;
  hwnd: number;
  icon?: string;
}

export interface PinnedWindow {
  hwnd: number;
  process_name: string;
  window_title: string;
  opacity: number;
  click_through: boolean;
  icon?: string;
}

export interface AppSettings {
  language: 'en' | 'es' | 'pt-br';
  start_with_windows: boolean;
  keep_app_on_top: boolean;
  refresh_interval_ms: number;
  excluded_processes: string[];
  hotkey_pin_toggle: string;
  default_opacity: number;
}

export interface PinnedImage {
  id: string;
  path: string;
  x: number;
  y: number;
  opacity: number;
  scale: number;
  click_through: boolean;
  /** basename extracted on the frontend for display */
  filename?: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  language: 'en',
  start_with_windows: false,
  keep_app_on_top: false,
  refresh_interval_ms: 5000,
  excluded_processes: [],
  hotkey_pin_toggle: 'Alt+P',
  default_opacity: 1,
};
