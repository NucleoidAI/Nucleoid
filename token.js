var Identifier = require("./identifier");

function isDelimiter(c) {
  return !(
    (48 <= c && c <= 57) ||
    (65 <= c && c <= 90) ||
    (97 <= c && c <= 122) ||
    c === 34 ||
    c === 36 ||
    c === 39 ||
    c === 46 ||
    c === 95
  );
}

const next = function (string, offset) {
  if (offset >= string.length) {
    return null;
  }

  let token = "";
  let active = false;
  let singleOn = false;
  let doubleOn = false;

  for (; offset < string.length; offset++) {
    let character = string.charCodeAt(offset);

    if (character === 10 || character === 13) {
      continue;
    }

    if (character === 96) {
      throw SyntaxError("Backtick is not supported.");
    }

    if (singleOn) {
      singleOn = character === 39 ? false : true;
      token += String.fromCharCode(character);
      continue;
    }

    if (doubleOn) {
      doubleOn = character === 34 ? false : true;
      token += String.fromCharCode(character);
      continue;
    }

    if (!isDelimiter(character) && active === false) {
      active = true;
      singleOn = character === 39 ? true : false;
      doubleOn = character === 34 ? true : false;
      token += String.fromCharCode(character);
    } else if (!isDelimiter(character) && active === true) {
      token += String.fromCharCode(character);
    } else if (isDelimiter(character) && active === true) {
      break;
    } else if (isDelimiter(character) && active === false) {
      if (character === 32) {
        continue;
      } else {
        token = String.fromCharCode(character);
      }

      offset++;
      break;
    }
  }

  return { token: token, offset: offset };
};

class Token {
  static check(string, offset) {
    let context = next(string, offset);
    return context.token;
  }

  static each(string, offset, callback, end) {
    let tokens = [];
    let context = next(string, offset);
    let parentheses = 0;
    let brackets = 0;

    while (context) {
      offset = context.offset;
      let token = context.token;

      if (brackets <= 0 && token === ";") {
        return { tokens: tokens, offset: offset };
      }

      if (end && token === end) {
        return { tokens: tokens, offset: offset };
      }

      if (token === "{") {
        brackets++;
      } else if (token === "}") {
        brackets--;
      }

      if (brackets < 0) {
        break;
      }

      if (token === "(") {
        parentheses++;
      } else if (token === ")") {
        parentheses--;
      }

      if (parentheses < 0) {
        break;
      }

      tokens.push(callback(token));
      context = next(string, offset);
    }

    return { tokens: tokens, offset: offset };
  }

  static nextBlock(string, offset) {
    let block = "";
    let brackets = 0;
    let character;

    for (; offset < string.length; offset++) {
      character = string.charAt(offset);

      if (character === "{") {
        brackets++;
      } else if (character === "}") {
        brackets--;
      }

      if (brackets < 0) {
        offset++;
        return { block: block, offset: offset };
      } else {
        block += character;
      }
    }

    throw SyntaxError("Missing parenthesis");
  }

  static nextArgs(string, offset) {
    let args = [];
    let context = next(string, offset);

    if (context.token === "}")
      throw SyntaxError(`Unexpected token ${context.token}`);

    while (context.token !== ")") {
      args.push(context.token);

      context = next(string, context.offset);
      if (context.token === ")") {
        break;
      } else {
        context = next(string, context.offset);
      }
    }

    return { args, offset: context.offset };
  }

  constructor(string) {
    this.string = string;
  }

  concat(string) {
    this.string = this.string.concat(string);
  }

  construct() {
    return this.string;
  }
}

module.exports = Token;
module.exports.next = next;
module.exports.ARRAY = class ARRAY {
  constructor() {
    this.length = 0;
  }

  construct() {
    let string = new String();

    for (let index = 0; index < this.length; index++) {
      string = string.concat(this[index].construct());
    }

    return string;
  }

  [Symbol.iterator]() {
    let list = this;
    let index = 0;

    return {
      next() {
        if (index < list.length) {
          return { done: false, value: list[index++] };
        } else {
          return { done: true };
        }
      },
    };
  }

  forEach(fn) {
    for (let index = 0; index < this.length; index++) fn(this[index].string);
  }

  map(fn) {
    let list = new ARRAY();

    for (let index = 0; index < this.length; index++) {
      let token = this[index];
      let string = fn(token.string);

      if (token instanceof CALL) {
        let params = [];

        for (let param of token.params) {
          if (param instanceof FUNCTION) {
            let f = new FUNCTION();
            f.short = param.short;
            f.params = param.params;
            f.block = [];

            for (let p of param.block) {
              if (param.params.includes(p)) {
                f.block.push(p);
              } else {
                f.block.push(fn(p));
              }
            }

            params.push(f);
          } else {
            params.push(fn(param));
          }
        }

        list.push(new CALL(string, params));
      } else {
        list.push(new Token(string));
      }
    }

    return list;
  }

  list() {
    let list = [];

    for (let index = 0; index < this.length; index++) {
      let token = this[index];

      if (token instanceof CALL) {
        let parts = Identifier.splitLast(token.string);

        if (parts[1] !== undefined) {
          list.push(parts[1]);
        }

        for (let param of token.params) {
          if (param instanceof FUNCTION) {
            list = list.concat(param.block);
          } else {
            list.push(param);
          }
        }
      } else {
        list.push(token.string);
      }
    }

    return list;
  }

  push(token) {
    this[this.length++] = token;
  }
};

class CALL extends Token {
  constructor(string, params) {
    super(string);
    this.params = params;
  }

  construct() {
    return this.string
      .concat("(")
      .concat(
        this.params
          .map((param) => {
            if (param instanceof FUNCTION) {
              return param.construct();
            }

            return param;
          })
          .join("")
      )
      .concat(")");
  }
}

class FUNCTION extends Token {
  constructor(short) {
    super();
    this.short = short;
  }

  construct() {
    if (this.short === true) {
      return "("
        .concat(this.params.join(","))
        .concat(")=>")
        .concat(this.block.join(""));
    }

    return "function"
      .concat("(")
      .concat(this.params.join(","))
      .concat(")")
      .concat(this.block.join(""));
  }
}

module.exports.CALL = CALL;
module.exports.FUNCTION = FUNCTION;
