export const ADMIN_BIT = 1 << 0;

export const permissionOptions = [
	{
		bit: ADMIN_BIT,
		label: "Admin",
		description: "Full access across Lyra, including all user management actions.",
	},
	{
		bit: 1 << 1,
		label: "Create invites",
		description: "Can issue invite links for pending accounts.",
	},
	{
		bit: 1 << 2,
		label: "Manage users",
		description: "Can create, edit, reset, and delete other accounts.",
	},
	{
		bit: 1 << 3,
		label: "Edit watch state",
		description: "Can update watch progress for other users.",
	},
	{
		bit: 1 << 4,
		label: "View all libraries",
		description: "Can see libraries regardless of narrower assignment rules later on.",
	},
] as const;
