import type { CodegenConfig } from "@graphql-codegen/cli";

const config: CodegenConfig = {
	overwrite: true,
	schema: "../schema.gql",
	documents: ["src/**/*.{ts,tsx}"],
	ignoreNoDocuments: true,
	generates: {
		"./src/@generated/gql/": {
			preset: "client",
			config: {
				// Use `unknown` instead of `any` for unconfigured scalars
				defaultScalarType: "unknown",
				// Apollo Client always includes `__typename` fields
				nonOptionalTypename: true,
				// Apollo Client doesn't add the `__typename` field to root types so
				// don't generate a type for the `__typename` for root operation types.
				skipTypeNameForRoot: true,
				useTypeImports: true,
				avoidOptionals: {
					// Use `null` for nullable fields instead of optionals
					field: true,
					// Allow nullable input fields to remain unspecified
					inputValue: false,
				},
			},
			presetConfig: {
				fragmentMasking: { unmaskFunctionName: "unmask" },
			},
		},
	},
};

export default config;
