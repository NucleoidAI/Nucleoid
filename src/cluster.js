const express = require("express");
const cors = require("cors");
const config = require("./config");
const axios = require("axios").default;

let extension;

try {
  extension = require(`${config.path.ext}/cluster.js`);
} catch (err) {
  console.error("Missing cluster extension");
  process.exit(1);
}

const cluster = express();
cluster.use(express.json());
cluster.use(cors());

cluster.all("*", async (req, res) => {
  const { ip, port, path } = extension.apply(req);

  try {
    const response = await axios({
      method: req.method,
      url: `http://${ip}:${port}${path}`,
      data: req.method !== "GET" ? req.body : undefined,
    });

    res.status(response.status).json(response.data);
  } catch (err) {
    res.status(503).end();
  }
});

cluster.listen(config.port.cluster);
