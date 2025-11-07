import express, { Request, Response } from "express";
import cors from "cors";
import config from "./config";
import axios, { AxiosRequestConfig, AxiosResponse } from "axios";

function init(): void {
  let extension: {
    apply: (req: Request) => {
      error?: boolean;
      https: boolean;
      ip: string;
      port: number;
      path: string;
      headers?: Record<string, string>;
    };
  };
  const _config: { path: string; port: { cluster: number } } = config();

  try {
    extension = require(`${_config.path}/extensions/cluster.js`);
  } catch (err) {
    console.error("Missing cluster extension");
    process.exit(1);
  }

  const cluster = express();
  cluster.use(express.json());
  cluster.use(cors());

  cluster.all("*", async (req: Request, res: Response) => {
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
      const response: AxiosResponse = await axios({
        method: req.method as AxiosRequestConfig["method"],
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

export { init };
