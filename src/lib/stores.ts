import { writable } from 'svelte/store';

export const rule = writable<null | [number, Date]>(null);
