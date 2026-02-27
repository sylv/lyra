import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import path from "node:path";
import webfontDownload from 'vite-plugin-webfont-dl';
import { tanstackRouter } from '@tanstack/router-plugin/vite'

export default defineConfig({
	plugins: [
		tanstackRouter({
			target: 'react',
			autoCodeSplitting: true,
			generatedRouteTree: 'src/@generated/routeTree.tsx',
		}),
		react({
			babel: {
				plugins: [["babel-plugin-react-compiler", { target: "19" }]],
			},
		}),
		webfontDownload([]),
		tailwindcss(),
	],
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),
		},
	},
	server: {
		proxy: {
			"/api": {
				target: "http://localhost:8000",
				changeOrigin: true,
			},
		},
	},
});
