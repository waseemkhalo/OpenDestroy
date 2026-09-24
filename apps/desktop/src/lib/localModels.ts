export type PublicLocalModel = {
  id: string;
  name: string;
  engine: string;
  language: string;
  downloadBytes: number;
  installed: boolean;
  supported: boolean;
  unavailableReason?: string;
};

export type PublicLocalModels = {
  selectedModelId: string | null;
  downloadedBytes: number;
  totalBytes: number;
  downloadingModelId: string | null;
  error?: string;
  availableBytes: number | null;
  models: PublicLocalModel[];
};

export const emptyPublicLocalModels = (): PublicLocalModels => ({
  selectedModelId: null,
  downloadedBytes: 0,
  totalBytes: 0,
  downloadingModelId: null,
  availableBytes: null,
  models: [],
});

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "Unknown size";
  if (bytes < 1024) return `${Math.round(bytes)} B`;
  const units = ["KiB", "MiB", "GiB", "TiB"];
  let value = bytes;
  let unit = -1;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value >= 10 || Number.isInteger(value) ? value.toFixed(0) : value.toFixed(1)} ${units[unit]}`;
}

export function modelProgress(snapshot: PublicLocalModels, model: PublicLocalModel): number | null {
  if (snapshot.downloadingModelId !== model.id || snapshot.totalBytes <= 0) return null;
  return Math.min(100, Math.max(0, Math.round((snapshot.downloadedBytes / snapshot.totalBytes) * 100)));
}

export function canRemoveModel(snapshot: PublicLocalModels, model: PublicLocalModel, busy: boolean): boolean {
  return !busy && model.installed && snapshot.downloadingModelId !== model.id;
}

export function freeSpaceMessage(snapshot: PublicLocalModels, model: PublicLocalModel): string {
  if (snapshot.availableBytes === null) return "Free space unavailable — check storage before downloading";
  if (snapshot.availableBytes >= model.downloadBytes) return `Available on this Mac: ${formatBytes(snapshot.availableBytes)}`;
  return `Not enough free space — needs ${formatBytes(model.downloadBytes - snapshot.availableBytes)} more`;
}
