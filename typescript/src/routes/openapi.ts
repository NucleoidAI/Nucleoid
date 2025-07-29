import express, { Request, Response, Router } from "express";
import openapi from "../lib/openapi";
import context from "../context";
import config from "../config";
import Joi from "joi";

const router: Router = express.Router();

router.get("/openapi", (req: Request, res: Response) => {
  res.json(openapi.status());
});

router.post("/openapi", (req: Request, res: Response) => {
  const {
    "x-nuc-action": action,
    "x-nuc-functions": functions = [],
    "x-nuc-declarations": declarations = [],
    "x-nuc-port": port,
  }: {
    "x-nuc-action": string;
    "x-nuc-functions"?: { definition: string }[];
    "x-nuc-declarations"?: { definition: string }[];
    "x-nuc-port"?: number;
  } = Joi.attempt(
    req.body,
    Joi.object({
      "x-nuc-action": Joi.string().required(),
      "x-nuc-functions": Joi.array().optional(),
      "x-nuc-declarations": Joi.array().optional(),
      "x-nuc-port": Joi.number().optional(),
    })
      .required()
      .options({ stripUnknown: true })
  );

  if (action === "start") {
    context.run(
      functions.map(({ definition }) => ({
        definition,
      }))
    );
    context.run(
      declarations.map(({ definition }) => ({
        definition,
        options: { declarative: true },
      }))
    );

    openapi.init();
    openapi.load(req.body);
  }

  res.end();
});

export default router;
