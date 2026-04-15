import { ADMIN_BIT, permissionOptions } from "./user-permissions";

export const describePermissions = (permissions: number) => {
  if ((permissions & ADMIN_BIT) !== 0) {
    return ["Admin"];
  }

  const labels = permissionOptions
    .filter((option) => option.bit !== ADMIN_BIT && (permissions & option.bit) !== 0)
    .map((option) => option.label);

  return labels.length > 0 ? labels : ["No extra permissions"];
};
