var next = (module.exports.next = function(string, offset) {
  if (offset >= string.length) {
    return null;
  }

  let token = "";
  let active = false;
  let stringOn = false;

  let isDelimiter = function(character) {
    return character === 32 ? true : false;
  };

  for (; offset < string.length; offset++) {
    let character = string.charCodeAt(offset);

    if (stringOn) {
      stringOn = character === 39 ? false : true;
      token += String.fromCharCode(character);
      continue;
    }

    if (!isDelimiter(character) && active === false) {
      active = true;
      stringOn = character === 39 ? true : false;
      token += String.fromCharCode(character);
    } else if (!isDelimiter(character) && active === true) {
      token += String.fromCharCode(character);
    } else if (isDelimiter(character) && active === true) {
      break;
    }
  }

  return { token: token, offset: offset };
});

module.exports.check = function(string, offset) {
  let context = next(string, offset);
  return context.token;
};

module.exports.each = function(string, offset, callback, end) {
  let tokens = [];
  let context = next(string, offset);
  let parentheses = 0;
  let brackets = 0;

  while (context) {
    offset = context.offset;
    let token = context.token;

    if (token === ";") {
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
};

module.exports.nextBlock = function(string, offset) {
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

  throw new SyntaxError(`Unexpected token ${character}`);
};
