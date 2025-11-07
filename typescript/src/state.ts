import { register } from "./transaction";
import serialize from "./lib/serialize";

// Scope type based on src/Scope.ts
interface Scope {
  prior: Scope | null;
  block: object;
  root: Scope;
  local: Record<string, unknown>;
  instance: object | null;
  graph: Record<string, unknown>;
  callback: Array<() => void>;
  instances: Record<string, unknown>;
  object?: { name: string };
  [key: string]: unknown;
}

// State shape
interface State {
  [key: string]: unknown;
  classes: unknown[];
}

const $: State = {
  classes: [],
};

// Evaluation type: Accepts string or { value: string }
type Evaluation = string | { value: string };

type SerializableValue =
  | string
  | number
  | boolean
  | null
  | undefined
  | Date
  | RegExp
  | Map<SerializableValue, SerializableValue>
  | Set<SerializableValue>
  | { id?: string; [key: string]: SerializableValue }
  | Array<SerializableValue>;

function assign(
  scope: Scope,
  variable: string,
  evaluation: Evaluation,
  json: boolean = true
): SerializableValue {
  let value: SerializableValue;
  if (json) {
    const evalValue = typeof evaluation === "string" ? evaluation : evaluation.value;
    value = serialize(eval(`(${evalValue})`), "state");
  } else {
    value = typeof evaluation === "string" ? evaluation : evaluation.value;
  }
  return register(`state.${variable}`, value) as SerializableValue;
}

function call(
  scope: Scope,
  fn: string,
  args: SerializableValue[] = []
): SerializableValue {
  const exec = `state.${fn}(${args.join(",")})`;
  return eval(exec) as SerializableValue;
}

function expression(
  scope: Scope,
  evaluation: { value: string }
): SerializableValue {
  return eval(`(${evaluation.value})`) as SerializableValue;
}

function del(scope: Scope, variable: string): boolean {
  return eval(`delete state.${variable}`);
}

function throwException(scope: Scope, exception: string): never {
  // This will throw and never return
  throw eval(`throw ${exception}`);
}

function clear(): void {
  for (const property in $) {
    delete $[property];
  }
  $["classes"] = [];
}

export {
  $,
  assign,
  call,
  expression,
  clear,
  del as delete,
  throwException as throw,
};
