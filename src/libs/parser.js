const Token = require("../utils/token");

const fn = (string) => {
  let args;
  let fn;
  let name;

  let context = Token.next(string, 0);

  if (context.token === "function") {
    const checkName = Token.next(string, context.offset);

    if (checkName && checkName.token !== "(") {
      context = checkName;
      name = checkName.token;
    }

    context = Token.next(string, context.offset);

    if (context && context.token !== "(") {
      throw SyntaxError(`Unexpected token '${context.token}'`);
    }

    context = Token.nextArgs(string, context.offset);
    args = context.args;

    const check = Token.next(string, context.offset);
    if (check && check.token !== "{") {
      throw SyntaxError(`Unexpected token '${check.token}'`);
    }
  } else if (context.token === "(") {
    context = Token.nextArgs(string, context.offset);
    args = context.args;
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
  } else {
    throw SyntaxError("Function expected");
  }

  const check = Token.next(string, context.offset);

  if (check && check.token === "{") {
    context = Token.nextBlock(string, check.offset);
    fn = `{${context.block.trim()}}`;
  } else {
    context = Token.nextBlock(string, context.offset, true);
    fn = `{return ${context.block.trim()}}`;
  }

  return { name, args, fn };
};

module.exports.fn = fn;
