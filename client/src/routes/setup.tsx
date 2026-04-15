import type { FC } from "react";
import { Outlet } from "react-router";
import { Modal } from "../components/modal";

export const SetupRoute: FC = () => (
  <Modal
    open={true}
    onOpenChange={() => {}}
    className="h-150 max-h-[85vh] w-225 max-w-[80vw]"
    style={{ aspectRatio: "auto" }}
  >
    <Outlet />
  </Modal>
);
