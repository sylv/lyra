import { z } from "zod";

type AnySchema = z.ZodTypeAny;

type UnwrappedInfo = {
	base: AnySchema;
	optional: boolean;
	nullable: boolean;
	hasDefault: boolean;
	defaultValue: unknown;
};

function unwrapModifiers(schema: AnySchema): UnwrappedInfo {
	let current: AnySchema = schema;

	let optional = false;
	let nullable = false;
	let hasDefault = false;
	let defaultValue: unknown = undefined;

	while (true) {
		if (current instanceof z.ZodOptional) {
			optional = true;
			current = (current as z.ZodOptional<AnySchema>).unwrap();
			continue;
		}

		if (current instanceof z.ZodNullable) {
			nullable = true;
			current = (current as z.ZodNullable<AnySchema>).unwrap();
			continue;
		}

		if (current instanceof z.ZodDefault) {
			hasDefault = true;
			defaultValue = current.parse(undefined);
			current = (current as z.ZodDefault<AnySchema>).unwrap();
			continue;
		}

		break;
	}

	return {
		base: current,
		optional,
		nullable,
		hasDefault,
		defaultValue,
	};
}

function preprocessLeafInput(raw: unknown, info: UnwrappedInfo): unknown {
	let v = raw;

	// Only rewrite empty string for optional/nullable fields.
	if (v === "" && (info.optional || info.nullable)) {
		v = undefined;
	}

	// optional: allow null -> undefined
	if (v === null && info.optional) {
		v = undefined;
	}

	// nullable-only: allow undefined/missing -> null
	if (v === undefined && info.nullable && !info.optional && !info.hasDefault) {
		v = null;
	}

	// Only coerce booleans if the base expects boolean.
	if (typeof v === "string" && info.base instanceof z.ZodBoolean) {
		if (v === "true") return true;
		if (v === "false") return false;
		return v;
	}

	// Only coerce numbers if the base expects number.
	if (typeof v === "string" && info.base instanceof z.ZodNumber) {
		const trimmed = v.trim();
		if (trimmed === "") return v;
		const n = Number(trimmed);
		return Number.isNaN(n) ? v : n;
	}

	return v;
}

function adaptQueryLeaf(schema: AnySchema): AnySchema {
	const info = unwrapModifiers(schema);

	if (!info.optional && !info.nullable && !info.hasDefault) {
		throw new Error("Each query field must use .optional(), .nullable(), or .default(...)");
	}

	let rebuilt: AnySchema = z.preprocess((raw) => preprocessLeafInput(raw, info), info.base);

	// Reapply wrappers to preserve output semantics.
	if (info.hasDefault) {
		rebuilt = rebuilt.default(info.defaultValue);
		rebuilt = rebuilt.catch(info.defaultValue);
		return rebuilt;
	}

	if (info.optional && info.nullable) {
		rebuilt = rebuilt.nullish();
		rebuilt = rebuilt.catch(undefined);
		return rebuilt;
	}

	if (info.optional) {
		rebuilt = rebuilt.optional();
		rebuilt = rebuilt.catch(undefined);
		return rebuilt;
	}

	if (info.nullable) {
		rebuilt = rebuilt.nullable();
		rebuilt = rebuilt.catch(null);
		return rebuilt;
	}

	return rebuilt;
}

function adaptQuerySchemaInner(schema: AnySchema): AnySchema {
	if (schema instanceof z.ZodObject) {
		const nextShape: Record<string, AnySchema> = {};
		for (const [key, value] of Object.entries(schema.shape)) {
			nextShape[key] = adaptQuerySchemaInner(value as AnySchema);
		}

		return z.object(nextShape);
	}

	if (schema instanceof z.ZodArray) {
		const arraySchema = schema as z.ZodArray<AnySchema>;
		return z.array(adaptQuerySchemaInner(arraySchema.element));
	}

	return adaptQueryLeaf(schema);
}

export function adaptQuerySchema<T extends AnySchema>(schema: T): T {
	return adaptQuerySchemaInner(schema) as T;
}
