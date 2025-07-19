export const getImageProxyUrl = (url: string, height: number): string => {
	return `/api/image-proxy/${encodeURIComponent(url)}?height=${height}`;
};
