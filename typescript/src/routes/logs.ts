import express, { Request, Response, Router } from "express";
import fs from "fs";
import path from "path";
import config from "../config";
import datastore from "../datastore";

const router: Router = express.Router();

router.get("/logs", (req: Request, res: Response) => {
  res.json(datastore.tail());
});

export default router;
