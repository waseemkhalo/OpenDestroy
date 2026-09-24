export type ConnectionOwner = Readonly<{
  expectedUserId?: string | null;
  expectedBackendUrl?: string;
  revision: number;
}>;

/** A UI callback may commit only while its connection and operation are current. */
export function isCurrentFencedWork(
  owner: ConnectionOwner,
  current: ConnectionOwner,
  expectedGeneration: number,
  currentGeneration: number,
  disposed: boolean,
): boolean {
  return !disposed
    && expectedGeneration === currentGeneration
    && owner.revision === current.revision
    && owner.expectedUserId === current.expectedUserId
    && owner.expectedBackendUrl === current.expectedBackendUrl;
}

export function practiceUsesLiveStream(provider: string | null | undefined): boolean {
  return provider === "backend";
}

export type LocalResetResult = Readonly<{ reset: boolean; modelsRemoved?: boolean }>;

export function localResetSucceeded(result: LocalResetResult): boolean {
  return result.reset === true;
}

/** Clear only Destroy's own browser-cache namespace after native reset succeeds. */
export function clearScopedLocalStorage(
  storage: Pick<Storage, "length" | "key" | "removeItem">,
  prefix = "destroy.",
): void {
  const keys: string[] = [];
  for (let index = 0; index < storage.length; index += 1) {
    const key = storage.key(index);
    if (key?.startsWith(prefix)) keys.push(key);
  }
  for (const key of keys) storage.removeItem(key);
}
