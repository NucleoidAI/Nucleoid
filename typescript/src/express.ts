import express, { Express, Request, Response, NextFunction } from "express";
import cors from "cors";
import fs from "fs";
import * as context from "./context";
import * as openapi from "./lib/openapi";

type AppOptions = Record<string, unknown>;

function app(options: AppOptions) {
  const app = express();
  app.use(express.json());
  app.use(cors());
  app.static = express.static;

  let listener: ReturnType<Express["listen"]>;

  return {
    express: (): Express => app,
    address: (): ReturnType<typeof listener.address> | null => (listener ? listener.address() : null),
    use: (...args: Parameters<Express["use"]>) => app.use(...args),
    get: (string: string, fn: (req: Request, res: Response) => void) => app.get(string, (req: Request, res: Response) => accept(req, res, fn)),
    post: (string: string, fn: (req: Request, res: Response) => void) => app.post(string, (req: Request, res: Response) => accept(req, res, fn)),
    put: (string: string, fn: (req: Request, res: Response) => void) => app.put(string, (req: Request, res: Response) => accept(req, res, fn)),
    delete: (string: string, fn: (req: Request, res: Response) => void) =>
      app.delete(string, (req: Request, res: Response) => accept(req, res, fn)),
    listen: (port: number = 3000, fn?: () => void) => {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const nucleoid = require("../");

      app.all("*", (req: Request, res: Response) => res.status(404).end());
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      app.use((err: Error, req: Request, res: Response, next: NextFunction) => res.status(500).send(err.stack));

      nucleoid.start(options);
      return (listener = app.listen(port, fn));
    },
    context: (path: string) => {
      const file = fs.readFileSync(path, "utf8");
      const arr = JSON.parse(file);
      context.load(
        arr.map((item: Record<string, unknown>) => ({ ...item, options: { declarative: true } }))
      );
    },
    openapi: (path: string) => {
      try {
        const file = fs.readFileSync(path, "utf8");
        openapi.init(app);
        openapi.load(JSON.parse(file));
      } catch (err) {
        throw Error("Problem occurred while opening OpenAPI");
      }
    },
  };
}

function accept(req: Request, res: Response, fn: (req: Request, res: Response) => void): void {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const nucleoid = require("../");

  const scope = { params: req.params, query: req.query, body: req.body };
  const { result, error } = nucleoid.run(fn, scope, { details: true });
  if (!result) res.status(404).end();
  else if (error) res.status(400).json(result);
  else res.status(200).json(result);
}

export default app;
