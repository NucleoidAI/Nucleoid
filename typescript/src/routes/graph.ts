import express, { Request, Response, Router } from "express";
import graph from "../lib/graph";

const router: Router = express.Router();

router.get("/", (req: Request, res: Response) => {
  const result: { [key: string]: { id: string; type: string; [prop: string]: string | string[] } } = {};

  for (const key in graph) {
    const node = graph[key];

    if (node.key) {
      result[node.key] = node;

      const tmp = {
        id: node.key,
        type: node.constructor.name,
      };

      for (const prop in node) {
        const object = node[prop];

        if (typeof object === "object" && object?.key) {
          tmp[prop] = object.key;
        } else if (typeof object === "object" && !object?.key) {
          const list = [];

          for (const innerProp in object) {
            const innerObject = object[innerProp];

            if (typeof innerObject === "object" && innerObject?.key) {
              list.push(innerObject.key);
            }
          }

          if (list.length) {
            tmp[prop] = list;
          }
        }
      }

      result[key] = tmp;
    }
  }

  res.json(result);
});

export default router;
