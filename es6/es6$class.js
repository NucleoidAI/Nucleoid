const Token = require("../token");
let $CLASS = require("../$class");
let ES6$BLOCK = require("./es6$block");

module.exports = function ES6$CLASS(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);
  let name = context.token;
  context = Token.next(string, context.offset);

  if (context === null || context.token === ";")
    throw SyntaxError("Missing parentheses");

  if (context.token !== "{")
    throw SyntaxError(`Unexpected token ${context.token}`);

  context = Token.nextBlock(string, context.offset);
  let block = context.block.trim();
  let construct;
  let args;

  if (block !== "") {
    let context = Token.next(block, 0);

    if (context.token === "constructor") {
      context = Token.next(block, context.offset);

      context = Token.nextArgs(block, context.offset);
      args = context.args;

      context = ES6$BLOCK(block, context.offset);
      construct = context.statement;
    } else {
      throw SyntaxError("Methods are not supported.");
    }

    context = Token.next(block, context.offset);

    if (context !== null && context.token !== "}") {
      throw SyntaxError("Methods are not supported.");
    }
  }

  return {
    statement: $CLASS(name, construct, args),
    offset: context.offset,
  };
};
