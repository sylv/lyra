export const getImageProxyUrl = (url: string, height: number): string => {
	if (url.startsWith("/api/assets/")) {
		const separator = url.includes("?") ? "&" : "?";
		return `${url}${separator}height=${height}`;
	}

	return `/api/image-proxy/${encodeURIComponent(url)}?height=${height}`;
};
