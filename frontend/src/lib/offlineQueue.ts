import { openDB, type DBSchema, type IDBPDatabase } from "idb";
import { api, OfflineError } from "./api";
import { notifyEntriesChanged } from "./events";
import type { CreateEntryInput, CreateProductionInput } from "./types";

interface QueuedEntry {
  id?: number;
  kind: "entry";
  exploitationId: string;
  input: CreateEntryInput;
  queuedAt: string;
}

interface QueuedProduction {
  id?: number;
  kind: "production";
  exploitationId: string;
  input: CreateProductionInput;
  queuedAt: string;
}

export type QueuedItem = QueuedEntry | QueuedProduction;

interface EleviaDB extends DBSchema {
  queue: {
    key: number;
    value: QueuedItem;
  };
}

let dbPromise: Promise<IDBPDatabase<EleviaDB>> | null = null;

function getDb() {
  if (!dbPromise) {
    dbPromise = openDB<EleviaDB>("elevia-offline", 1, {
      upgrade(db) {
        db.createObjectStore("queue", { keyPath: "id", autoIncrement: true });
      },
    });
  }
  return dbPromise;
}

export async function queueEntry(
  exploitationId: string,
  input: CreateEntryInput,
): Promise<void> {
  const db = await getDb();
  await db.add("queue", {
    kind: "entry",
    exploitationId,
    input,
    queuedAt: new Date().toISOString(),
  });
}

export async function queueProduction(
  exploitationId: string,
  input: CreateProductionInput,
): Promise<void> {
  const db = await getDb();
  await db.add("queue", {
    kind: "production",
    exploitationId,
    input,
    queuedAt: new Date().toISOString(),
  });
}

export async function pendingCount(): Promise<number> {
  const db = await getDb();
  return db.count("queue");
}

let flushing = false;

/**
 * Retries every queued submission against the API, removing each one that
 * succeeds. Entries/production are upserts server-side (keyed on
 * exploitation/month[/product]), so resubmitting is always safe. Safe to call
 * repeatedly - a no-op while offline or already flushing.
 */
export async function flushQueue(): Promise<void> {
  if (flushing || !navigator.onLine) return;
  flushing = true;
  try {
    const db = await getDb();
    const items = await db.getAll("queue");
    for (const item of items) {
      try {
        if (item.kind === "entry") {
          await api.submitEntry(item.exploitationId, item.input);
        } else {
          await api.submitProduction(item.exploitationId, item.input);
        }
        if (item.id !== undefined) {
          await db.delete("queue", item.id);
        }
        if (item.kind === "entry") {
          notifyEntriesChanged();
        }
      } catch (err) {
        if (err instanceof OfflineError) {
          // Network dropped again mid-flush - stop here, the next trigger retries the rest.
          break;
        }
        // Any other error (e.g. a stale product): leave it queued and retry next time
        // rather than silently discarding the farmer's data.
        console.error("failed to sync queued item, will retry later", item, err);
      }
    }
  } finally {
    flushing = false;
  }
}

/** Wires up automatic retry: on reconnect, and every 30s while the app is open. */
export function startAutoSync(): void {
  window.addEventListener("online", () => {
    void flushQueue();
  });
  setInterval(() => void flushQueue(), 30_000);
  void flushQueue();
}
