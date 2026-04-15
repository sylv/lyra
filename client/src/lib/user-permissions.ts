export const ADMIN_BIT = 1 << 0;
export const CREATE_INVITE_BIT = 1 << 1;
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
    bit: EDIT_OTHERS_WATCH_STATE_BIT,
    label: "Edit watch state",
    description: "Can update watch progress for other users.",
  },
] as const;
