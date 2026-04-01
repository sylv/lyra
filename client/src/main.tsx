import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router";
import { App } from "./app";
import "./globals.css";
import "@fontsource-variable/inter/wght.css";

const rootElement = document.getElementById("app");
if (!rootElement) {
	throw new Error("Missing #app root element");
}

createRoot(rootElement).render(
	<BrowserRouter>
		<App />
	</BrowserRouter>,
);
