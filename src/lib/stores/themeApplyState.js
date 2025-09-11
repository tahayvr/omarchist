import { writable } from 'svelte/store';

// A global writable store to hold the applying theme directory, or null if no theme is being applied
export const themeApplyState = writable(null);
