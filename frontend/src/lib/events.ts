// EntryForm and EntryList are separate Astro/Svelte islands (each hydrated
// independently via client:load) with no shared props, so a successful
// submission in one has no way to tell the other to refetch. A plain
// window event is the simplest bridge between islands for this.
const ENTRIES_CHANGED = "elevia:entries-changed";

export function notifyEntriesChanged(): void {
  window.dispatchEvent(new Event(ENTRIES_CHANGED));
}

export function onEntriesChanged(handler: () => void): () => void {
  window.addEventListener(ENTRIES_CHANGED, handler);
  return () => window.removeEventListener(ENTRIES_CHANGED, handler);
}
