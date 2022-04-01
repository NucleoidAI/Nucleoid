const { argv } = require("yargs");
const fs = require("fs");
const File = require("../file");
const path = File.data;

const PROCESS_PATH = `${path}/${argv.id || "main"}`;

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
};

module.exports = { tail, clear };
