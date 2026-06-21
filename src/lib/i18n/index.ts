import { derived } from 'svelte/store';
import { settingsStore } from '$lib/stores/settings.store';
import en, { type Translations } from './en';
import es from './es';
import ptBr from './pt-br';

const locales: Record<string, Translations> = { en, es, 'pt-br': ptBr };

export const t = derived(settingsStore, ($s) => locales[$s.language] ?? en);
