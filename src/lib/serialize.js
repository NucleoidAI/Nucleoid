let state;

setImmediate(() => (state = require("../state").state));

function serialize(input) {
  let result;

  switch (true) {
    case input instanceof Array:
      result = `[${input.map(serialize).join(",")}]`;
      break;
    case input instanceof Map:
      result = `new Map(${serialize([...input])})`;
      break;
    case input instanceof Set:
      result = `new Set(${serialize([...input])})`;
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
      if (input.id !== undefined && state[input.id] !== undefined) {
        result = `state.${input.id}`;
        break;
      }
      result = `{${Object.entries(input)
        .map(([key, value]) => `${key}:${serialize(value)}`)
        .join(",")}}`;
      break;
    default:
      result = JSON.stringify(input);
  }

  return result;
}

module.exports = serialize;
