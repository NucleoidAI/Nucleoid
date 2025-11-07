import express, { Request, Response, Router } from "express";
import os from "os";
import metrics from "../lib/metrics";

const router: Router = express.Router();

router.get("/metrics", (req: Request, res: Response) => {
  res.json({
    free: os.freemem(),
    total: os.totalmem(),
  });
});

export default router;
