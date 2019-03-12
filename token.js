var next = (module.exports.next = function(string, offset) {
  if (offset >= string.length) {
    return null;
  }

  let token = "";
  let active = false;

  let isDelimiter = function(character) {
    return character == 32 ? true : false;
  };

  for (; offset < string.length; offset++) {
    let character = string.charCodeAt(offset);

    if (!isDelimiter(character) && active == false) {
      active = true;
      token += String.fromCharCode(character);
    } else if (!isDelimiter(character) && active == true) {
      token += String.fromCharCode(character);
    } else if (isDelimiter(character) && active == true) {
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
  let tokens = "";
  let context = next(string, offset);
  let parentheses = 0;

  while (context) {
    offset = context.offset;
    let token = context.token;

    if (token == ";" || token == "}") {
      return { tokens: tokens, offset: offset };
    }

    if (end && token == end) {
      return { tokens: tokens, offset: offset };
    }

    if (token == "(") {
      parentheses++;
    } else if (token == ")") {
      parentheses--;
    }

    if (parentheses < 0) {
      break;
    }

    tokens += callback(token);
    context = next(string, offset);
  }

  return { tokens: tokens, offset: offset };
};

module.exports.nextBlock = function(string, offset) {
  let block = "";

  for (; offset < string.length; offset++) {
    let character = string.charAt(offset);

    if (character != "}") {
      block += character;
    } else {
      break;
    }
  }

  offset++;
  return { block: block, offset: offset };
};
