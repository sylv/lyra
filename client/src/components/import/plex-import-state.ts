export type PlexImportStep =
  | "connect"
  | "authenticating"
  | "selectServer"
  | "fetching"
  | "checking"
  | "conflicts"
  | "confirm"
  | "importing"
  | "complete"
  | "error";

export interface PlexServerOption {
  id: string;
  name: string;
  baseUrl: string;
  accessToken: string;
  protocol: "http" | "https";
  isRelay: boolean;
  isLocal: boolean;
}

export interface PlexImportWatchStateRow {
  source: string;
  sourceItemId?: string | null;
  title?: string | null;
  mediaType?: string | null;
  seasonNumber?: number | null;
  episodeNumber?: number | null;
  progressPercent: number;
  viewedAt?: number | null;
  filePath?: string | null;
  fileBasename?: string | null;
  fileSizeBytes?: number | null;
  imdbId?: string | null;
  tmdbId?: number | null;
}

export interface PlexImportCompatibility {
  matchedRows: number;
  unmatchedRows: number;
  conflictRows: number;
  willInsert: number;
  willOverwrite: number;
}

export interface PlexImportState {
  step: PlexImportStep;
  errorMessage: string | null;
  accountToken: string | null;
  servers: PlexServerOption[];
  selectedServerId: string | null;
  rows: PlexImportWatchStateRow[];
  compatibility: PlexImportCompatibility | null;
  overwriteConflicts: boolean;
  importedCount: number;
  skippedCount: number;
}

export const FETCHING_METADATA_TEXT = "Fetching metadata, one sec...";

export const INITIAL_PLEX_IMPORT_STATE: PlexImportState = {
  step: "connect",
  errorMessage: null,
  accountToken: null,
  servers: [],
  selectedServerId: null,
  rows: [],
  compatibility: null,
  overwriteConflicts: false,
  importedCount: 0,
  skippedCount: 0,
};

export const getNextStepFromCompatibility = (compatibility: PlexImportCompatibility): PlexImportStep => {
  if (compatibility.conflictRows > 0) {
    return "conflicts";
  }

  return "confirm";
};
