export const getImageProxyUrl = (url: string, height: number): string => {
	const separator = url.includes("?") ? "&" : "?";
	return `${url}${separator}height=${height}`;
};
