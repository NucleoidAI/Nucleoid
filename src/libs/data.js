const { argv } = require("yargs");
const fs = require("fs");
const File = require("../file");
const path = File.data;

const PROCESS_PATH = `${path}/${argv.id}`;

const retrieve = () => {
  if (fs.existsSync(PROCESS_PATH)) {
    try {
      const data = fs.readFileSync(PROCESS_PATH, "utf8");
      const buffer = data.split("\n").reverse().slice(0, 11).join("\n");
      return Buffer.from(buffer).toString("base64");
    } catch (error) {
      throw Error("Cannot retrieve data");
    }
  } else {
    return Buffer.from("").toString("base64");
  }
};

const clear = () => {
  fs.rmdirSync(path, { recursive: true, force: true });
};

module.exports = { retrieve, clear };
