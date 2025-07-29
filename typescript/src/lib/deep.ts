// Deprecated in favor of lodash
function deepEqual(obj1: object, obj2: object): boolean {
  if (obj1 === obj2) {
    return true;
  }

  if (
    typeof obj1 !== "object" ||
    obj1 === null ||
    typeof obj2 !== "object" ||
    obj2 === null
  ) {
    return false;
  }

  const keys1 = Object.keys(obj1);
  const keys2 = Object.keys(obj2);

  if (keys1.length !== keys2.length) {
    return false;
  }

  for (const key of keys1) {
    if (!keys2.includes(key) || !deepEqual(obj1[key], obj2[key])) {
      return false;
    }
  }

  return true;
}

function deepMerge(target: object, source: object): object {
  const output = { ...target };
  for (const key in source) {
    // eslint-disable-next-line no-prototype-builtins
    if (source.hasOwnProperty(key)) {
      if (_isObject(source[key]) && _isObject(target[key])) {
        output[key] = deepMerge(target[key], source[key]);
      } else {
        output[key] = source[key];
      }
    }
  }
  return output;
}

//eslint-disable-next-line @typescript-eslint/no-explicit-any
function equal(actual: any, expected: any): boolean {
  return actual === expected;
}

function _isObject(item: unknown): item is object {
  return Boolean(item && typeof item === "object" && !Array.isArray(item));
}

export { deepEqual, equal, deepMerge };
