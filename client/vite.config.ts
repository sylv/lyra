import { babelOptimizerPlugin } from '@graphql-codegen/client-preset';
import babel from '@rolldown/plugin-babel';
import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from '@tanstack/router-plugin/vite';
import react, { reactCompilerPreset } from '@vitejs/plugin-react';
import * as child from 'child_process';
import { readFileSync } from "node:fs";
import path from "node:path";
import { defineConfig } from "vite";
import codegen from 'vite-plugin-graphql-codegen';
import webfontDownload from 'vite-plugin-webfont-dl';

function getVariables() {
	let revision = 'unknown';
	let branch = 'unknown';
	try {
		revision = child.execSync('git describe --tags --always --dirty').toString().trim();
		branch = child.execSync('git rev-parse --abbrev-ref HEAD').toString().trim();
	} catch { }

	const buildTime = new Date().toISOString();
	return { revision, buildTime, branch };
}


const { revision, buildTime, branch } = getVariables();
export default defineConfig({
	plugins: [
		codegen(),
		tanstackRouter({
			target: 'react',
			autoCodeSplitting: true,
			generatedRouteTree: 'src/@generated/routeTree.tsx',
		}),
		react(),
		babel({
			plugins: [[babelOptimizerPlugin, { artifactDirectory: './src/@generated/gql/', gqlTagName: 'graphql' }]],
			presets: [reactCompilerPreset()],
			include: /\.(tsx|jsx)$/,
		}),
		webfontDownload([]),
		tailwindcss(),
	],
	resolve: {
		tsconfigPaths: true,
	},
	build: {
		sourcemap: true
	},
	server: {
		port: 3000,
		proxy: {
			"/api": {
				target: "http://localhost:8000",
				changeOrigin: true,
				ws: true,
			},
		},
	},
	define: {
		__REVISION__: JSON.stringify(revision),
		__BRANCH__: JSON.stringify(branch),
		__BUILD_TIME__: JSON.stringify(buildTime),
	}
});
