var graph = require("./state").state.graph;

module.exports = class Statement {
  constructor(string) {
    this.string = string;
    this.tokens = "";
    this.token = "";
    this.offset = 0;
  }

  next(checkFlag) {
    if (this.markFlag) {
      this.token = "state." + this.token;
      this.markFlag = false;
    }

    if (this.skipFlag) {
      this.skipFlag = false;
    } else {
      this.tokens += this.token;
    }

    if (this.offset >= this.string.length) {
      return null;
    }

    let token = "";
    let active = false;
    let count = 0;

    let isDelimiter = function(character) {
      return character == 32 ? true : false;
    };

    for (; this.offset < this.string.length; this.offset++, count++) {
      let character = this.string.charCodeAt(this.offset);

      if (!isDelimiter(character) && active == false) {
        active = true;
        token += String.fromCharCode(character);
      } else if (!isDelimiter(character) && active == true) {
        token += String.fromCharCode(character);
      } else if (isDelimiter(character) && active == true) {
        break;
      }
    }

    if (checkFlag) {
      this.offset -= count;
    } else {
      this.token = token;
    }

    return token;
  }

  mark() {
    this.markFlag = true;
  }

  skip() {
    this.skipFlag = true;
    return this.next();
  }

  check() {
    this.skipFlag = true;
    return this.next(true);
  }

  each(callback, end) {
    let tokens = "";

    while (this.next()) {
      if (this.token == ";" || this.token == "}") {
        return tokens;
      }

      if (end && this.token == end) {
        return tokens;
      }

      if (callback(this.token)) {
        this.mark();
        tokens = "state." + this.token;
      } else {
        tokens += this.token;
      }
    }

    return tokens;
  }

  nextBlock() {
    let block = "";

    for (; this.offset < this.string.length; this.offset++) {
      let character = this.string.charAt(this.offset);

      if (character != "}") {
        block += character;
      } else {
        break;
      }
    }

    this.offset++;
    return block;
  }

  toString() {
    return this.tokens;
  }

  run() {
    if (graph[this.token]) {
      this.mark();
    }

    this.each(token => graph[token]);
  }
};
