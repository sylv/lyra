export type InitState =
	| { state: "login" }
	| { state: "create_first_user" }
	| { state: "create_invited_user"; invite_code: string; username: string }
	| { state: "create_first_library" }
	| { state: "ready" };

export type PendingInitState = Exclude<InitState, { state: "ready" }>;
export type SetupStepRoute = "/setup/login" | "/setup/create-account" | "/setup/create-library";

export const fetchInitState = async (searchStr = ""): Promise<InitState> => {
	const params = new URLSearchParams(searchStr);
	const inviteCode = params.get("inviteCode");
	const initUrl = inviteCode?.trim() ? `/api/init?inviteCode=${encodeURIComponent(inviteCode)}` : "/api/init";
	const response = await fetch(initUrl);

	if (!response.ok) {
		throw new Error(`Failed to load setup state (${response.status})`);
	}

	return response.json();
};

export const isSetupReady = (state: InitState): state is Extract<InitState, { state: "ready" }> =>
	state.state === "ready";

export const isSetupPath = (pathname: string) => pathname === "/setup" || pathname.startsWith("/setup/");

export const getSetupRouteForState = (state: PendingInitState): SetupStepRoute => {
	switch (state.state) {
		case "login":
			return "/setup/login";
		case "create_first_user":
		case "create_invited_user":
			return "/setup/create-account";
		case "create_first_library":
			return "/setup/create-library";
	}
};

export const getRelativeLocationUri = ({
	pathname,
	searchStr = "",
	hash = "",
}: {
	pathname: string;
	searchStr?: string;
	hash?: string;
}) => `${pathname}${searchStr}${hash}`;

export const getSetupRedirectUri = (state: PendingInitState, previous: string) =>
	`${getSetupRouteForState(state)}?${new URLSearchParams({
		previous,
		...(state.state === "create_invited_user" ? { inviteCode: state.invite_code } : {}),
	}).toString()}`;

export const getPreviousSetupRoute = (searchStr: string) => {
	const previous = new URLSearchParams(searchStr).get("previous");

	if (!previous || !previous.startsWith("/") || previous.startsWith("//")) {
		return "/";
	}

	if (previous === "/setup" || previous.startsWith("/setup/") || previous.startsWith("/setup?")) {
		return "/";
	}

	return previous;
};
