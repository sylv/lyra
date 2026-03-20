import prettyMilliseconds from "pretty-ms";
import { ADMIN_BIT, permissionOptions } from "./types";

export const describePermissions = (permissions: number) => {
	if ((permissions & ADMIN_BIT) !== 0) {
		return ["Admin"];
	}

	const labels = permissionOptions
		.filter((option) => option.bit !== ADMIN_BIT && (permissions & option.bit) !== 0)
		.map((option) => option.label);

	return labels.length > 0 ? labels : ["No extra permissions"];
};

export const formatLastSeen = (lastSeenAt?: number | null) => {
	if (!lastSeenAt) {
		return "Not signed in yet";
	}

	const seenAt = new Date(lastSeenAt * 1000);
	const now = new Date();

	if (
		seenAt.getFullYear() === now.getFullYear() &&
		seenAt.getMonth() === now.getMonth() &&
		seenAt.getDate() === now.getDate()
	) {
		return "Last seen today";
	}

	const elapsedMs = Math.max(0, now.getTime() - seenAt.getTime());
	return `Last seen ${prettyMilliseconds(elapsedMs, { unitCount: 1, verbose: true })} ago`;
};
