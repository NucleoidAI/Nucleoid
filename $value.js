var Token = require("./token");
var EXPRESSION = require("./expression");
var $ = require("./$");
var graph = require("./graph");
var REFERENCE = require("./reference");
var EXPRESSION$CLASS = require("./expression$class");

module.exports = function(string, offset) {
  let context = Token.each(string, offset, function(token) {
    if (token === "new") {
      return "new ";
    }

    if (token === "return") {
      return "return ";
    }

    if (token === "typeof") {
      return "typeof ";
    }

    if (token === "instanceof") {
      return " instanceof ";
    }

    return token;
  });

  let list = new Token.ARRAY();
  let tokens = context.tokens;

  for (let i = 0; i < tokens.length; i++) {
    if (tokens[i + 1] === "(") {
      let call = new Token.CALL(tokens[i]);

      let params = [];
      let parentheses = 1;
      i += 2;

      for (; i < tokens.length; i++) {
        let token = tokens[i];

        if (token === "(") {
          parentheses++;
        } else if (token === ")") {
          parentheses--;
        }

        if (parentheses <= 0) {
          break;
        }

        if (token === "function") {
          let fn = new Token.FUNCTION();

          let context = getParams(tokens, ++i);
          fn.params = context.params;

          context = getBlock(tokens, context.offset);
          fn.block = context.block;

          params.push(fn);
          i = context.offset;
        } else if (tokens[i + 1] === "=>") {
          let fn = new Token.FUNCTION(true);

          fn.params = [token];
          i += 2;

          token = tokens[i];

          if (token === "{") {
            let context = getBlock(tokens, i);
            fn.block = context.block;

            params.push(fn);
            i = context.offset;
          } else {
            let context = untilBlock(tokens, i);
            fn.block = context.block;

            params.push(fn);
            i = context.offset;
          }
        } else {
          params.push(token);
        }
      }

      if (tokens[i + 1] && tokens[i + 1].charAt(0) === ".") {
        throw new SyntaxError("Nested functions are not supported.");
      }

      call.params = params;
      list.push(call);
    } else {
      list.push(new Token(tokens[i]));
    }
  }

  if (context.tokens.length === 0) {
    return { offset: context.offset };
  }

  let statement = new $VALUE();
  statement.tokens = list;

  return { statement: statement, offset: context.offset };
};

class $VALUE extends $ {
  run() {
    if (this.tokens.length === 1 && graph[this.tokens[0].string]) {
      let statement = new REFERENCE();
      statement.link = graph[this.tokens[0].string];
      return statement;
    } else {
      for (let token of this.tokens.list()) {
        let prefix = token.split(".")[0];

        if (graph[prefix] && graph[prefix].instanceof === "CLASS") {
          let statement = new EXPRESSION$CLASS();
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
