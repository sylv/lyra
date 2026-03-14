import { thumbHashToDataURL } from "thumbhash";

const HEX_PATTERN = /^[0-9a-f]+$/i;

export const getThumbhashDataUrl = (thumbhashHex: string | null | undefined): string | null => {
	if (!thumbhashHex || thumbhashHex.length % 2 !== 0 || !HEX_PATTERN.test(thumbhashHex)) {
		return null;
	}

	const bytes = new Uint8Array(thumbhashHex.length / 2);
	for (let i = 0; i < thumbhashHex.length; i += 2) {
		bytes[i / 2] = Number.parseInt(thumbhashHex.slice(i, i + 2), 16);
	}

	try {
		return thumbHashToDataURL(bytes);
	} catch {
		return null;
	}
};
