import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import path from "node:path";
import webfontDownload from 'vite-plugin-webfont-dl';
import { tanstackRouter } from '@tanstack/router-plugin/vite'
import codegen from 'vite-plugin-graphql-codegen';
import { babelOptimizerPlugin } from '@graphql-codegen/client-preset'

export default defineConfig({
	plugins: [
		codegen(),
		tanstackRouter({
			target: 'react',
			autoCodeSplitting: true,
			generatedRouteTree: 'src/@generated/routeTree.tsx',
		}),
		react({
			babel: {
				plugins: [
					["babel-plugin-react-compiler", { target: "19" }],
					[babelOptimizerPlugin, { artifactDirectory: './src/@generated/gql/', gqlTagName: 'graphql' }]
				],
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
		port: 3000,
		proxy: {
			"/api": {
				target: "http://localhost:8000",
				changeOrigin: true,
			},
		},
	},
});
