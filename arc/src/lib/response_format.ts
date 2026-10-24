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

export const code: { type: "string" } = {
  type: "string",
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
