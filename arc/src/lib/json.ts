export function isObject(value: unknown): value is {
  [property: string]: unknown;
} {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
