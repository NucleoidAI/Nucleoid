const nucleoid = require("../../index");
const Process = require("./process");
const config = require("../config");
const parser = require("../libs/parser");
const { post } = require("axios").default;

let cluster;

try {
  cluster = require(`${config.path.handlers}/cluster.js`);
} catch (err) {} // eslint-disable-line no-empty

async function run(req, res, action) {
  let fn;

  try {
    fn = parser.fn(action.toString()).fn;
  } catch (err) {
    res.status(400).json({
      message: "Invalid function syntax",
    });
  }

  let process;

  if (cluster) {
    process = cluster(req, fn);
  } else {
    process = req.headers["process"];
  }

  const scope = { process };
  const { ip, port } = nucleoid.run((scope) => Process[scope.process], scope);

  try {
    const { data } = await post(`http://${ip}:${port}`, fn, {
      headers: {
        "content-type": "application/javascript",
        process,
      },
    });
    res.json(data);
  } catch (err) {
    res.status(503).end();
  }
}

module.exports = run;
