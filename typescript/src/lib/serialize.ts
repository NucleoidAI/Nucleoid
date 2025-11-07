/**
 * Serializes JavaScript values to a string representation.
 * Handles circular references by using a reference system.
 *
 * @param value - The value to serialize
 * @param source - Source identifier for reference tracking
 * @param refs - Internal parameter used for tracking object references
 * @returns A string representation of the value
 */
function serialize(
  value: unknown,
  source: string,
  refs: Map<object, string> = new Map()
): string {
  // Handle null
  if (value === null) {
    return "null";
  }

  // Handle undefined
  if (value === undefined) {
    return "undefined";
  }

  const valueType = typeof value;

  // Handle primitives
  if (valueType === "string") {
    return JSON.stringify(value);
  }

  if (valueType === "number" || valueType === "boolean") {
    return String(value);
  }

  if (valueType === "function") {
    return value.toString();
  }

  // Handle objects
  if (valueType === "object") {
    // Check for circular references
    if (refs.has(value as object)) {
      return `{$ref:{id:'${refs.get(value as object)}',source:'${source}'}}`;
    }

    // Handle arrays
    if (Array.isArray(value)) {
      return JSON.stringify(value);
    }

    // Handle Date
    if (value instanceof Date) {
      return `new Date(${value.getTime()})`;
    }

    // Handle RegExp
    if (value instanceof RegExp) {
      return value.toString();
    }

    // Handle Map
    if (value instanceof Map) {
      const entries = Array.from(value.entries()).map(([k, v]) => [
        JSON.stringify(k),
        serialize(v, source, refs),
      ]);
      return `new Map([${entries.map(([k, v]) => `[${k},${v}]`).join(",")}])`;
    }

    // Handle Set
    if (value instanceof Set) {
      const values = Array.from(value).map((v) => serialize(v, source, refs));
      return `new Set([${values.join(",")}])`;
    }

    // Handle regular objects
    if (value.constructor === Object) {
      // Store the reference ID for this object
      const obj = value as Record<string, unknown>;
      if (obj.hasOwnProperty("id")) {
        refs.set(value as object, String(obj.id));
      }

      const entries = Object.entries(value).map(
        ([k, v]) => `${k}:${serialize(v, source, refs)}`
      );
      return `{${entries.join(",")}}`;
    }

    // Handle other object types
    return `[object ${value.constructor.name}]`;
  }

  // Handle other types
  return String(value);
}

export default serialize;
