type VariableTransaction = { variable: string; before: unknown };
type ObjectTransaction = {
  object: Record<string, any>;
  property: string;
  before: unknown;
};
type Transaction = VariableTransaction | ObjectTransaction;

let list: Transaction[] = [];

export function start(): void {
  list = [];
}

export function end(): Transaction[] {
  const result = list;
  list = [];
  return result;
}

export function register(variable: string, value: unknown): unknown;
export function register(
  object: Record<string, any>,
  property: string,
  value: unknown
): void;

export function register(
  p1: string | Record<string, any> | undefined,
  p2?: unknown,
  p3?: unknown
): unknown | void {
  if (p1 === undefined) return;

  if (typeof p1 === "string") {
    const variable = p1;
    const value = p2;
    // capture old value
    // eslint-disable-next-line no-eval
    const before = eval(variable);
    // apply new value
    // eslint-disable-next-line no-eval
    const after = eval(`${variable} = value`);
    list.push({ variable, before });
    return after;
  } else {
    // --- Object transaction ---
    const object = p1;
    const property = p2 as string;
    const value = p3;
    const before = object[property];
    list.push({ object, property, before });
    object[property] = value;
  }
}

export function rollback(): void {
  while (list.length > 0) {
    const transaction = list.pop()!;

    if ("variable" in transaction) {
      // global variable case
      // eslint-disable-next-line no-eval
      eval(`${transaction.variable} = transaction.before`);
    } else {
      const { object, property, before } = transaction;
      if (before === undefined) {
        delete object[property];
      } else {
        object[property] = before;
      }
    }
  }
}
