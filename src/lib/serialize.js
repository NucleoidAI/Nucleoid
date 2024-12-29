function serialize(input, source, acc = {}) {
  const state = require("../state").$;
  const graph = require("../graph").$;
  let result;

  switch (true) {
    case input instanceof Array:
      result = `[${input.map((i) => serialize(i, source, acc)).join(",")}]`;
      break;
    case input instanceof Map:
      result = `new Map(${serialize([...input], source, acc)})`;
      break;
    case input instanceof Set:
      result = `new Set(${serialize([...input], source, acc)})`;
      break;
    case input instanceof Date:
      result = `new Date(${input.getTime()})`;
      break;
    case input instanceof String:
    case input instanceof Number:
    case input instanceof Boolean:
      result = JSON.stringify(input);
      break;
    case input instanceof Function:
    case input instanceof RegExp:
      result = input.toString().replace(/\n/g, " ").replace(/ +/g, " ");
      break;
    case input instanceof Object:
      if (source === "state" && state[input?.id] !== undefined) {
        result = `state.${input.id}`;
        break;
      } else if (source === "graph" && graph[input?.id] !== undefined) {
        result = `graph['${input.id}']`;
        break;
      } else if (input.id !== undefined && acc[input.id] !== undefined) {
        result = `{$ref:{id:'${input.id}',source:'${source}'}}`;
        break;
      } else {
        if (input.id !== undefined) {
          acc[input.id] = input;
        }
      }

      result = `{${Object.entries(input)
        .map(([key, value]) => `${key}:${serialize(value, source, acc)}`)
        .join(",")}}`;
      break;
    default:
      result = JSON.stringify(input);
  }

  return result;
}

module.exports = serialize;
