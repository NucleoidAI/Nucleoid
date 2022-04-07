const Token = require("../utils/token");

const fn = (string) => {
  let args;
  let fn;

  let context = Token.next(string, 0);

  if (context.token === "function") {
    context = Token.next(string, context.offset);
    context = Token.nextArgs(string, context.offset);
    args = context.args;
  } else if (context.token === "(") {
    context = Token.nextArgs(string, context.offset);
    args = context.args;
    context = Token.next(string, context.offset);
    context = Token.next(string, context.offset);
  } else {
    args = [context.token];
  }

  let check = Token.next(string, context.offset);

  if (check && check.token === "{") {
    context = Token.nextBlock(string, check.offset);
    fn = `{${context.block.trim()}}`;
  } else {
    context = Token.nextBlock(string, context.offset, true);
    fn = context.block.trim();
  }

  return { args, fn };
};

module.exports.fn = fn;
