const fs = require("fs");
const File = require("../file");
const path = File.data;
const process = require("../process");

const PROCESS_PATH = `${path}/${process.options().id}`;

const tail = () => {
  if (fs.existsSync(PROCESS_PATH)) {
    try {
      const data = fs.readFileSync(PROCESS_PATH, "utf8");
      return data
        .split("\n")
        .reverse()
        .slice(1)
        .map((data) => JSON.parse(data));
    } catch (error) {
      throw Error("Cannot retrieve data");
    }
  } else {
    return [];
  }
};

const clear = () => {
  fs.rmSync(path, { recursive: true, force: true });
  fs.mkdirSync(path, { recursive: true });
};

module.exports = { tail, clear };
