import type { PlexImportWatchStateRow, PlexServerOption } from "./plex-import-state";

const PLEX_PIN_ENDPOINT = "https://plex.tv/api/v2/pins";
const PLEX_RESOURCES_ENDPOINT = "https://plex.tv/api/v2/resources";
const PLEX_AUTH_URL = "https://app.plex.tv/auth";
const PLEX_IMPORT_CLIENT_ID_KEY = "lyra_plex_import_client_id";
const PLEX_IMPORT_CLIENT_NAME = "Lyra";
const PLEX_IMPORT_CLIENT_VERSION = "0.1";
const PLEX_PAGE_SIZE = 200;
const PLEX_AUTH_TIMEOUT_MS = 180_000;
const PLEX_AUTH_POLL_INTERVAL_MS = 2_000;

interface PlexPinResponse {
  id?: number;
  code?: string;
  authToken?: string | null;
}

interface PlexResourceConnection {
  protocol?: string;
  uri?: string;
  relay?: boolean;
  local?: boolean;
}

interface PlexResource {
  name?: string;
  clientIdentifier?: string;
  provides?: string;
  accessToken?: string;
  connections?: PlexResourceConnection[];
  Connection?: PlexResourceConnection[];
}

interface PlexMediaContainer {
  totalSize?: number;
  size?: number;
  Metadata?: unknown[];
  Directory?: unknown[];
}

const getPlexClientId = (): string => {
  const fromStorage = window.localStorage.getItem(PLEX_IMPORT_CLIENT_ID_KEY);
  if (fromStorage) {
    return fromStorage;
  }

  // crypto.randomUUID is not supported in some contexts (e.g, http)
  const generated = `lyra-${Math.random().toString(16).slice(2)}`;
  window.localStorage.setItem(PLEX_IMPORT_CLIENT_ID_KEY, generated);
  return generated;
};

const getPlexHeaders = (token?: string): HeadersInit => {
  const clientId = getPlexClientId();
  return {
    Accept: "application/json",
    "X-Plex-Client-Identifier": clientId,
    "X-Plex-Product": PLEX_IMPORT_CLIENT_NAME,
    "X-Plex-Version": PLEX_IMPORT_CLIENT_VERSION,
    "X-Plex-Device": "Browser",
    "X-Plex-Platform": "Web",
    ...(token ? { "X-Plex-Token": token } : {}),
  };
};

const getPlexPin = async (): Promise<PlexPinResponse> => {
  const response = await fetch(`${PLEX_PIN_ENDPOINT}?strong=true`, {
    method: "POST",
    headers: getPlexHeaders(),
  });

  if (!response.ok) {
    throw new Error("Unable to create Plex auth PIN");
  }

  return (await response.json()) as PlexPinResponse;
};

const buildPlexAuthUrl = (code: string) => {
  const params = new URLSearchParams({
    clientID: getPlexClientId(),
    code,
    "context[device][product]": PLEX_IMPORT_CLIENT_NAME,
    "context[device][version]": PLEX_IMPORT_CLIENT_VERSION,
  });

  // Plex expects auth params in a hash-query (`#?...`). Without the leading `?`,
  // app.plex.tv routes to the homepage and ignores the PIN flow context.
  return `${PLEX_AUTH_URL}#?${params.toString()}`;
};

const sleep = (delayMs: number) =>
  new Promise<void>((resolve) => {
    window.setTimeout(resolve, delayMs);
  });

const pollForPlexToken = async (pinId: number, popup: Window | null): Promise<string> => {
  const startedAt = Date.now();
  while (Date.now() - startedAt <= PLEX_AUTH_TIMEOUT_MS) {
    if (popup && popup.closed) {
      throw new Error("Plex authentication popup was closed");
    }

    const response = await fetch(`${PLEX_PIN_ENDPOINT}/${pinId}`, {
      headers: getPlexHeaders(),
    });
    if (!response.ok) {
      throw new Error("Failed to poll Plex PIN");
    }

    const payload = (await response.json()) as PlexPinResponse;
    if (payload.authToken) {
      return payload.authToken;
    }

    await sleep(PLEX_AUTH_POLL_INTERVAL_MS);
  }

  throw new Error("Timed out waiting for Plex authentication");
};

export const authenticateWithPlexPopup = async (): Promise<string> => {
  const pin = await getPlexPin();
  if (!pin.id || !pin.code) {
    throw new Error("Plex PIN response was incomplete");
  }

  const popup = window.open(buildPlexAuthUrl(pin.code), "lyra-plex-auth", "popup=yes,width=680,height=760");
  if (!popup) {
    throw new Error("Plex popup was blocked by the browser");
  }

  try {
    const accountToken = await pollForPlexToken(pin.id, popup);
    return accountToken;
  } finally {
    popup.close();
  }
};

const normalizeProtocol = (value: string | undefined): "http" | "https" | null => {
  if (!value) {
    return null;
  }

  const normalized = value.toLowerCase();
  if (normalized === "http" || normalized === "https") {
    return normalized;
  }

  return null;
};

const parsePlexResources = (payload: unknown): PlexResource[] => {
  if (Array.isArray(payload)) {
    return payload as PlexResource[];
  }

  if (
    payload &&
    typeof payload === "object" &&
    "MediaContainer" in payload &&
    payload.MediaContainer &&
    typeof payload.MediaContainer === "object" &&
    Array.isArray((payload.MediaContainer as { Device?: unknown[] }).Device)
  ) {
    return (payload.MediaContainer as { Device: PlexResource[] }).Device;
  }

  return [];
};

export const discoverPlexServers = async (accountToken: string, pageProtocol: string): Promise<PlexServerOption[]> => {
  const response = await fetch(`${PLEX_RESOURCES_ENDPOINT}?includeHttps=1&includeRelay=1`, {
    headers: getPlexHeaders(accountToken),
  });
  if (!response.ok) {
    throw new Error("Failed to discover Plex Media Servers");
  }

  const payload = await response.json();
  const resources = parsePlexResources(payload);
  const allowHttp = pageProtocol !== "https:";
  const optionsById = new Map<string, PlexServerOption>();

  for (const resource of resources) {
    const provides = (resource.provides ?? "").toLowerCase();
    if (!provides.includes("server")) {
      continue;
    }

    const connections = resource.connections ?? resource.Connection ?? [];
    for (const connection of connections) {
      const protocol = normalizeProtocol(connection.protocol);
      if (!protocol) {
        continue;
      }
      if (protocol === "http" && !allowHttp) {
        continue;
      }

      const baseUrl = (connection.uri ?? "").trim().replace(/\/$/, "");
      if (!baseUrl) {
        continue;
      }

      const serverId = resource.clientIdentifier ?? `${resource.name}-${baseUrl}`;
      const optionId = `${serverId}:${baseUrl}`;
      optionsById.set(optionId, {
        id: optionId,
        name: resource.name?.trim() || "Plex Server",
        baseUrl,
        accessToken: (resource.accessToken ?? accountToken).trim(),
        protocol,
        isRelay: Boolean(connection.relay),
        isLocal: Boolean(connection.local),
      });
    }
  }

  return [...optionsById.values()].sort((a, b) => {
    if (a.name !== b.name) {
      return a.name.localeCompare(b.name);
    }
    if (a.protocol !== b.protocol) {
      return a.protocol === "https" ? -1 : 1;
    }
    if (a.isRelay !== b.isRelay) {
      return a.isRelay ? 1 : -1;
    }
    return a.baseUrl.localeCompare(b.baseUrl);
  });
};

const fetchPlexPmsJson = async (
  server: PlexServerOption,
  path: string,
  params: Record<string, string> = {},
): Promise<PlexMediaContainer> => {
  const url = new URL(path, `${server.baseUrl}/`);
  url.searchParams.set("X-Plex-Token", server.accessToken);
  for (const [key, value] of Object.entries(params)) {
    url.searchParams.set(key, value);
  }

  const response = await fetch(url.toString(), {
    headers: getPlexHeaders(server.accessToken),
  });
  if (!response.ok) {
    throw new Error(`Plex API request failed: ${path}`);
  }

  const payload = await response.json();
  if (
    payload &&
    typeof payload === "object" &&
    "MediaContainer" in payload &&
    payload.MediaContainer &&
    typeof payload.MediaContainer === "object"
  ) {
    return payload.MediaContainer as PlexMediaContainer;
  }

  return payload as PlexMediaContainer;
};

const toNumber = (value: unknown): number | null => {
  if (typeof value === "number" && Number.isFinite(value)) {
    return value;
  }
  if (typeof value === "string" && value.trim() !== "") {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }
  return null;
};

const toStringValue = (value: unknown): string | null => {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
};

const basenameFromPath = (path: string | null): string | null => {
  if (!path) {
    return null;
  }

  const value = path.trim();
  if (!value) {
    return null;
  }

  const basename = value.split(/[\\/]/).pop()?.trim() ?? "";
  return basename || null;
};

const parseGuidIds = (metadata: Record<string, unknown>) => {
  const ids = new Set<string>();

  const guidField = metadata.guid;
  if (typeof guidField === "string") {
    ids.add(guidField);
  }

  const guidEntries = metadata.Guid;
  if (Array.isArray(guidEntries)) {
    for (const entry of guidEntries) {
      if (entry && typeof entry === "object" && "id" in entry) {
        const id = (entry as { id?: unknown }).id;
        if (typeof id === "string") {
          ids.add(id);
        }
      }
    }
  }

  const idsArray = [...ids];
  let imdbId: string | null = null;
  let tmdbId: number | null = null;

  for (const id of idsArray) {
    if (!imdbId) {
      const imdbMatch = id.match(/imdb:\/\/(tt\d+)/i);
      if (imdbMatch) {
        imdbId = imdbMatch[1];
      }
    }

    if (tmdbId == null) {
      const tmdbMatch = id.match(/tmdb:\/\/(\d+)/i);
      if (tmdbMatch) {
        tmdbId = Number.parseInt(tmdbMatch[1], 10);
      }
    }
  }

  return { imdbId, tmdbId };
};

const readFirstPart = (
  metadata: Record<string, unknown>,
): { filePath: string | null; fileSizeBytes: number | null } => {
  const mediaEntries = Array.isArray(metadata.Media) ? metadata.Media : [];
  for (const media of mediaEntries) {
    if (!media || typeof media !== "object") {
      continue;
    }

    const partEntries = Array.isArray((media as { Part?: unknown[] }).Part) ? (media as { Part: unknown[] }).Part : [];
    for (const part of partEntries) {
      if (!part || typeof part !== "object") {
        continue;
      }
      const filePath = toStringValue((part as { file?: unknown }).file);
      const fileSizeBytes = toNumber((part as { size?: unknown }).size);
      return { filePath, fileSizeBytes };
    }
  }

  return { filePath: null, fileSizeBytes: null };
};

const toImportRow = (metadata: Record<string, unknown>): PlexImportWatchStateRow | null => {
  const duration = toNumber(metadata.duration);
  const viewOffset = toNumber(metadata.viewOffset);
  const viewCount = toNumber(metadata.viewCount);

  let progressPercent: number | null = null;
  if (duration != null && duration > 0 && viewOffset != null) {
    progressPercent = Math.max(0, Math.min(1, viewOffset / duration));
  }

  if ((progressPercent == null || progressPercent <= 0) && viewCount != null && viewCount > 0) {
    progressPercent = 1;
  }

  if (progressPercent == null || progressPercent <= 0) {
    return null;
  }

  const { imdbId, tmdbId } = parseGuidIds(metadata);
  const { filePath, fileSizeBytes } = readFirstPart(metadata);
  const fileBasename = basenameFromPath(filePath);
  const sourceItemId = toStringValue(metadata.ratingKey);
  const mediaType = toStringValue(metadata.type);
  const title = toStringValue(metadata.title);

  return {
    source: "plex",
    sourceItemId,
    title,
    mediaType,
    seasonNumber: toNumber(metadata.parentIndex),
    episodeNumber: toNumber(metadata.index),
    progressPercent,
    viewedAt: toNumber(metadata.lastViewedAt),
    filePath,
    fileBasename,
    fileSizeBytes,
    imdbId,
    tmdbId,
  };
};

const fetchSectionRows = async (
  server: PlexServerOption,
  sectionKey: string,
  plexType: "1" | "4",
): Promise<PlexImportWatchStateRow[]> => {
  let start = 0;
  const rows: PlexImportWatchStateRow[] = [];

  while (true) {
    const container = await fetchPlexPmsJson(server, `/library/sections/${sectionKey}/all`, {
      type: plexType,
      includeGuids: "1",
      "X-Plex-Container-Start": String(start),
      "X-Plex-Container-Size": String(PLEX_PAGE_SIZE),
    });

    const metadataEntries = Array.isArray(container.Metadata) ? container.Metadata : [];
    for (const entry of metadataEntries) {
      if (!entry || typeof entry !== "object") {
        continue;
      }
      const row = toImportRow(entry as Record<string, unknown>);
      if (row) {
        rows.push(row);
      }
    }

    const fetched = metadataEntries.length;
    const total = toNumber(container.totalSize) ?? start + fetched;
    start += fetched;

    if (fetched === 0 || start >= total) {
      break;
    }
  }

  return rows;
};

const fetchPlexSections = async (server: PlexServerOption): Promise<Record<string, unknown>[]> => {
  const container = await fetchPlexPmsJson(server, "/library/sections");
  const sections = Array.isArray(container.Directory) ? container.Directory : [];
  return sections.filter(
    (section): section is Record<string, unknown> => section != null && typeof section === "object",
  );
};

export const fetchPlexWatchStateRows = async (server: PlexServerOption): Promise<PlexImportWatchStateRow[]> => {
  const sections = await fetchPlexSections(server);
  const rows: PlexImportWatchStateRow[] = [];

  for (const section of sections) {
    const sectionKey = toStringValue(section.key);
    const sectionType = toStringValue(section.type);
    if (!sectionKey || !sectionType) {
      continue;
    }

    if (sectionType === "movie") {
      rows.push(...(await fetchSectionRows(server, sectionKey, "1")));
    } else if (sectionType === "show") {
      rows.push(...(await fetchSectionRows(server, sectionKey, "4")));
    }
  }

  return rows;
};
