const Token = require("../../lib/token");
const EXPRESSION = require("../../nuc/EXPRESSION");
const $ = require("./$");
const graph = require("../../graph");
const REFERENCE = require("../../nuc/REFERENCE");
const EXPRESSION$INSTANCE = require("../../nuc/EXPRESSION$INSTANCE");
const Local = require("../../lib/local");

const operators = [
  "+",
  "-",
  "*",
  "/",
  "%",
  "++",
  "--",
  "=",
  "+=",
  "-=",
  "*=",
  "/=",
  "%=",
  "**=",
  "<<=",
  ">>=",
  ">>>=",
  "&=",
  "^=",
  "|=",
  "==",
  "!=",
  "===",
  "!==",
  ">",
  "<",
  ">=",
  "<=",
  "&&",
  "||",
  "!",
  "&",
  "|",
  "^",
  "~",
  "<<",
  ">>",
  ">>>",
  "?",
  ":",
  "...",
  "..?",
  "void",
  "typeof",
  "instanceof",
  "[",
  "]",
];

function construct(string, offset = 0) {
  let context = Token.each(string, offset);

  let list = new Token.ARRAY();
  let tokens = context.tokens;

  for (let i = 0; i < tokens.length; i++) {
    if (tokens[i + 1] === "(") {
      let context = parseCall(tokens, i);
      list.push(context.call);
      i = context.offset;

      let chained = tokens[i];

      while (chained?.charAt(0) === ".") {
        if (tokens[i + 1] === "(") {
          let context = parseCall(tokens, i);
          list.push(context.call);

          i = context.offset;
          chained = tokens[i];
        } else {
          list.push(new Token(chained));
          chained = tokens[++i];
        }
      }

      i--;
    } else {
      if (tokens[i] === "{") {
        const context = parseBrackets(tokens, i);
        i = context.offset;
        const bracket = new Token.BRACKET();
        context.tokens.forEach((token) => bracket.push(token));
        list.push(bracket);
      } else if (operators.includes(tokens[i].trim())) {
        list.push(new Token(tokens[i]));
      } else if (graph[tokens[i]]) {
        list.push(new Token.VARIABLE(tokens[i]));
      } else {
        list.push(new Token.EXPRESSION(tokens[i]));
      }
    }
  }

  if (context.tokens.length === 0) {
    return { offset: context.offset };
  }

  let statement = new $EXPRESSION();
  statement.tokens = list;

  return { statement: statement, offset: context.offset };
}

function parseCall(tokens, offset) {
  let call = new Token.CALL(tokens[offset]);

  let params = [];
  let parentheses = 1;
  offset += 2;

  let token = tokens[offset];

  if (token === "(") {
    parentheses++;
  } else if (token === ")") {
    parentheses--;
  }

  if (parentheses <= 0) {
    call.params = params;
    return { call, offset: ++offset };
  }

  if (token === "function") {
    let fn = new Token.FUNCTION();

    let context = getParams(tokens, ++offset);
    fn.params = context.params;

    context = getBlock(tokens, context.offset);
    fn.block = context.block;

    params.push(fn);
    offset = context.offset + 1;
  } else if (tokens[offset + 1] === "=" && tokens[offset + 2] === ">") {
    let fn = new Token.FUNCTION(true);

    fn.params = [token];
    offset += 3;

    token = tokens[offset];

    if (token === "{") {
      let context = getBlock(tokens, offset);
      fn.block = context.block;

      params.push(fn);
      offset = context.offset + 1;
    } else {
      let context = untilBlock(tokens, offset);
      fn.block = context.block;

      params.push(fn);
      offset = context.offset;
    }
  } else {
    let context = untilBlock(tokens, offset);
    offset = context.offset;

    if (context.block) {
      context.block.forEach((e) => params.push(e));
    }
  }

  call.params = params;
  return { call, offset: ++offset };
}

function parseBrackets(tokens, offset = 0) {
  let bracket = 0;
  let arr = [];

  for (let i = offset; i < tokens.length; i++) {
    let token = tokens[i];

    if (token === "{") {
      bracket++;
    } else if (token === "}") {
      bracket--;
    }

    arr.push(token);

    if (bracket === 0) {
      return { tokens: arr, offset: i };
    }
  }
}

class $EXPRESSION extends $ {
  run(scope) {
    if (
      this.tokens.length === 1 &&
      !(this.tokens[0] instanceof Token.CALL) &&
      graph[this.tokens[0].string] &&
      !Local.check(scope, this.tokens[0].string)
    ) {
      let statement = new REFERENCE();
      statement.link = graph[this.tokens[0].string];
      statement.tokens = this.tokens;
      return statement;
    } else {
      for (let token of this.tokens.list()) {
        if (token instanceof Token.BRACKET) {
          continue;
        }

        let prefix = token.split(/\.|\[|\]/)[0];

        if (graph[prefix] && graph[prefix].instanceof === "CLASS") {
          let statement = new EXPRESSION$INSTANCE();
          statement.class = graph[prefix];
          statement.tokens = this.tokens;
          return statement;
        }
      }

      let statement = new EXPRESSION();
      statement.tokens = this.tokens;
      return statement;
    }
  }
}

function getParams(tokens, offset) {
  let parentheses = 0;
  let params = [];

  for (++offset; offset < tokens.length; offset++) {
    let token = tokens[offset];

    if (token === "(") {
      parentheses++;
      continue;
    } else if (token === ")") {
      parentheses--;
      continue;
    }

    if (parentheses < 0) {
      break;
    }

    params.push(token);
  }

  return { params, offset };
}

function getBlock(tokens, offset) {
  let brackets = 0;
  let block = [];

  for (offset; offset < tokens.length; offset++) {
    let token = tokens[offset];

    if (token === "{") {
      brackets++;
    } else if (token === "}") {
      brackets--;
    }

    block.push(token);

    if (brackets <= 0) {
      break;
    }
  }

  return { block, offset };
}

function untilBlock(tokens, offset) {
  let brackets = 1;
  let parentheses = 1;
  let block = [];

  for (offset; offset < tokens.length; offset++) {
    let token = tokens[offset];

    if (token === "{") brackets++;
    else if (token === "}") brackets--;
    else if (token === "(") parentheses++;
    else if (token === ")") parentheses--;

    if (brackets <= 0 || parentheses <= 0) {
      break;
    }

    block.push(token);
  }

  return { block, offset };
}

module.exports = construct;
