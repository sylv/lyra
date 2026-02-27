import { RouterProvider } from "@tanstack/react-router";
import { createRoot } from "react-dom/client";
import { router } from "./router";
import "./globals.css";

const rootElement = document.getElementById("app");
if (!rootElement) {
	throw new Error("Missing #app root element");
}

createRoot(rootElement).render(<RouterProvider router={router} />);
