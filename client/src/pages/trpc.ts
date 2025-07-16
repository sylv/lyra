import { createTRPCReact } from "@trpc/react-query";
import type { AppRouter } from "../@generated/server";

export const trpc = createTRPCReact<AppRouter>();
