const express = require("express");
const cors = require("cors");
const config = require("./config");
const axios = require("axios").default;

function init() {
  let extension;
  const _config = config();

  try {
    extension = require(`${_config.path}/extensions/cluster.js`);
  } catch (err) {
    console.error("Missing cluster extension");
    process.exit(1);
  }

  const cluster = express();
  cluster.use(express.json());
  cluster.use(cors());

  cluster.all("*", async (req, res) => {
    const {
      error = true,
      https,
      ip,
      port,
      path,
      headers = {},
    } = extension.apply(req);

    if (error) {
      return res.status(403).end();
    }

    try {
      const response = await axios({
        method: req.method,
        url: `${https ? "https" : "http"}://${ip}:${port}${path}`,
        headers,
        data: req.method !== "GET" ? req.body : undefined,
      });

      res.status(response.status).json(response.data);
    } catch (err) {
      res.status(503).end();
    }
  });

  cluster.listen(_config.port.cluster);
}

module.exports = { init };
