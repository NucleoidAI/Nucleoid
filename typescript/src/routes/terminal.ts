import express, { NextFunction, Request, Response } from "express";

import { ValidationError } from "joi";
import cors from "cors";
import graph from "./graph";
import logs from "./logs";
import metrics from "./metrics";
import openapi from "./openapi";
import runtime from "../runtime";

const terminal = express();
terminal.use(cors());

terminal.post(
  "/",
  (req: Request, res: Response, next: NextFunction) =>
    req.is("application/javascript") ? next() : res.status(415).end(),
  express.text({ type: "application/javascript" }),
  (req: Request, res: Response) => {
    const details = runtime.process(req.body, { details: true });
    res.send(details);
  }
);

terminal.use(
  express.json(),
  (err: Error, req: Request, res: Response, next: NextFunction) =>
    err ? res.status(422).end() : next()
);
terminal.use(graph);
terminal.use(openapi);
terminal.use(logs);
terminal.use(metrics);

// eslint-disable-next-line no-unused-vars
terminal.use(
  (
    err: Error | string | { error: any },
    req: Request,
    res: Response,
    next: NextFunction
  ) => {
    if (typeof err === "string") {
      return res.status(400).json({ error: err });
    }

    if (err instanceof ValidationError) {
      return res.status(400).json({ message: err.message });
    }

    if ("error" in err) {
      res.status(400).json(err);
    } else {
      res.status(500).send(err.toString());
    }
  }
);

export default terminal;
