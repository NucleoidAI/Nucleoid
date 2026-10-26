import { isObject } from "./json";

export function object(properties: {
  [property: string]: { [keyword: string]: unknown };
}): {
  type: "object";
  properties: { [property: string]: { [keyword: string]: unknown } };
  required: string[];
  additionalProperties: false;
} {
  return {
    type: "object",
    properties,
    required: Object.keys(properties),
    additionalProperties: false,
  };
}

export const code: { type: "string"; minLength: number } = {
  type: "string",
  minLength: 1,
};

export function grid(
  rows?: number,
  cols?: number
): {
  type: "array";
  minItems: number;
  maxItems: number;
  items: {
    type: "array";
    minItems: number;
    maxItems: number;
    items: { type: "integer"; minimum: number; maximum: number };
  };
} {
  return {
    type: "array",
    minItems: rows ?? 1,
    maxItems: rows ?? 30,
    items: {
      type: "array",
      minItems: cols ?? 1,
      maxItems: cols ?? 30,
      items: { type: "integer", minimum: 0, maximum: 9 },
    },
  };
}

export function create(
  name: string,
  properties: { [property: string]: { [keyword: string]: unknown } }
): {
  type: "json_schema";
  json_schema: {
    name: string;
    strict: true;
    schema: {
      type: "object";
      properties: { [property: string]: { [keyword: string]: unknown } };
      required: string[];
      additionalProperties: false;
    };
  };
} {
  return {
    type: "json_schema",
    json_schema: {
      name,
      strict: true,
      schema: object(properties),
    },
  };
}

export function validate<Response>(
  value: unknown,
  schema: { [keyword: string]: unknown },
  errors: { path: string; message: string }[],
  path: string = ""
): value is Response {
  const fail = (message: string): false => {
    errors.push({ path, message });
    return false;
  };
  for (const keyword of Object.keys(schema)) {
    if (
      ![
        "type",
        "properties",
        "required",
        "additionalProperties",
        "items",
        "minItems",
        "maxItems",
        "minLength",
        "minimum",
        "maximum",
      ].includes(keyword)
    ) {
      throw new Error(`Unsupported response schema keyword: ${keyword}`);
    }
  }

  switch (schema.type) {
    case "object": {
      if (!isObject(value)) return fail("Expected an object");
      const properties = schema.properties;
      const required = schema.required;
      if (
        !isObject(properties) ||
        !Array.isArray(required) ||
        !required.every(
          (key: unknown): key is string => typeof key === "string"
        ) ||
        schema.additionalProperties !== false
      ) {
        throw new Error(`Invalid object response schema at ${path || "/"}`);
      }
      for (const key of required) {
        if (!Object.hasOwn(value, key)) {
          return fail(`Missing required property ${JSON.stringify(key)}`);
        }
      }
      for (const [key, item] of Object.entries(value)) {
        if (!Object.hasOwn(properties, key)) {
          return fail(`Unknown property ${JSON.stringify(key)}`);
        }
        const field = properties[key];
        if (!isObject(field)) {
          throw new Error(`Invalid response schema at ${path}/${key}`);
        }
        if (!validate(item, field, errors, `${path}/${key}`)) return false;
      }
      return true;
    }
    case "array": {
      if (!Array.isArray(value)) return fail("Expected an array");
      if (
        typeof schema.minItems === "number" &&
        value.length < schema.minItems
      ) {
        return fail(`Expected at least ${schema.minItems} items`);
      }
      if (
        typeof schema.maxItems === "number" &&
        value.length > schema.maxItems
      ) {
        return fail(`Expected at most ${schema.maxItems} items`);
      }
      const items = schema.items;
      if (!isObject(items)) {
        throw new Error(`Invalid array response schema at ${path || "/"}`);
      }
      return value.every((item: unknown, index: number) =>
        validate(item, items, errors, `${path}/${index}`)
      );
    }
    case "string":
      if (typeof value !== "string") return fail("Expected a string");
      if (
        typeof schema.minLength === "number" &&
        Array.from(value).length < schema.minLength
      ) {
        return fail(`Expected at least ${schema.minLength} characters`);
      }
      return true;
    case "number":
    case "integer":
      if (
        typeof value !== "number" ||
        !Number.isFinite(value) ||
        (schema.type === "integer" && !Number.isInteger(value))
      ) {
        return fail(`Expected a finite ${schema.type}`);
      }
      if (typeof schema.minimum === "number" && value < schema.minimum) {
        return fail(
          `Expected a number greater than or equal to ${schema.minimum}`
        );
      }
      if (typeof schema.maximum === "number" && value > schema.maximum) {
        return fail(
          `Expected a number less than or equal to ${schema.maximum}`
        );
      }
      return true;
    default:
      throw new Error(
        `Unsupported response schema type: ${String(schema.type)}`
      );
  }
}
