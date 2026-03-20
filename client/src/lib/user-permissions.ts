export const ADMIN_BIT = 1 << 0;
export const CREATE_INVITE_BIT = 1 << 1;
export const CREATE_USER_BIT = 1 << 2;
export const EDIT_OTHERS_WATCH_STATE_BIT = 1 << 3;
export const VIEW_ALL_LIBRARIES_BIT = 1 << 4;

export const permissionOptions = [
	{
		bit: ADMIN_BIT,
		label: "Admin",
		description: "Full access across Lyra, including all user management actions.",
	},
	{
		bit: CREATE_INVITE_BIT,
		label: "Create invites",
		description: "Can issue invite links for pending accounts.",
	},
	{
		bit: CREATE_USER_BIT,
		label: "Manage users",
		description: "Can create, edit, reset, and delete other accounts.",
	},
	{
		bit: EDIT_OTHERS_WATCH_STATE_BIT,
		label: "Edit watch state",
		description: "Can update watch progress for other users.",
	},
	{
		bit: VIEW_ALL_LIBRARIES_BIT,
		label: "View all libraries",
		description: "Can see libraries regardless of narrower assignment rules later on.",
	},
] as const;
