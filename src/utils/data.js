const { argv } = require("yargs");
const fs = require("fs");

const PROCESS_PATH = `${argv.path}/${argv.id}`;

module.exports.retrieve = () => {
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
