export function object(properties) {
  return {
    type: "object",
    properties,
    required: Object.keys(properties),
    additionalProperties: false,
  };
}

export const code = {
  type: "string",
  minLength: 1,
};

export function grid(rows?: number, cols?: number) {
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

export function create(name: string, properties) {
  return {
    type: "json_schema",
    json_schema: {
      name,
      strict: true,
      schema: object(properties),
    },
  };
}
