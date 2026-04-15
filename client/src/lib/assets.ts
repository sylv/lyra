export const appendAssetQuery = (
  signedUrl: string,
  params: Record<string, string | number | null | undefined>,
): string => {
  const searchParams = new URLSearchParams();

  for (const [key, value] of Object.entries(params)) {
    if (value == null) {
      continue;
    }

    searchParams.set(key, String(value));
  }

  const query = searchParams.toString();
  return query.length > 0 ? `${signedUrl}?${query}` : signedUrl;
};
