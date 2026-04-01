import { babelOptimizerPlugin } from "@graphql-codegen/client-preset";
import babel from "@rolldown/plugin-babel";
import tailwindcss from "@tailwindcss/vite";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import codegen from "vite-plugin-graphql-codegen";

function readBuildVariable(name: string) {
	const value = process.env[name]?.trim();
	return value ? value : "unknown";
}

const revision = readBuildVariable("LYRA_BUILD_REVISION");
const branch = readBuildVariable("LYRA_BUILD_BRANCH");
const buildTime = new Date().toISOString();

export default defineConfig({
	plugins: [
		codegen(),
		react(),
		babel({
			plugins: [[babelOptimizerPlugin, { artifactDirectory: "./src/@generated/gql/", gqlTagName: "graphql" }]],
			presets: [reactCompilerPreset()],
			include: /\.(tsx|jsx)$/,
		}),
		tailwindcss(),
	],
	resolve: {
		tsconfigPaths: true,
	},
	build: {
		sourcemap: true,
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
	preview: {
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
	},
});
