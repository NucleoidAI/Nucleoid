import { execFile } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import Ajv from "ajv";

const execFileAsync = promisify(execFile);
const validateSnapshot = new Ajv({ allErrors: true }).compile<{
  [property: string]: unknown;
}>({
  type: "object",
  additionalProperties: true,
});

async function execute(args: string[]) {
  const executable: string =
    process.env.NUCLEOID_EXECUTABLE ??
    String.raw`C:\Users\canmi\dev\Nucleoid\target\debug\nucleoid.exe`;
  const { stdout } = await execFileAsync(executable, args, {
    encoding: "utf8",
    windowsHide: true,
  }).catch((error: unknown) => {
    const stderr =
      error instanceof Error &&
      "stderr" in error &&
      typeof error.stderr === "string"
        ? error.stderr.trim()
        : "";
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Nucleoid execution failed: ${stderr || message}`, {
      cause: error,
    });
  });
  return stdout;
}

export async function version() {
  const stdout: string = await execute(["--version"]);
  const match = /^nucleoid\s+(\S+)\s*$/.exec(stdout);
  if (!match) {
    throw new Error("Nucleoid returned invalid version output");
  }
  return match[1];
}

export async function run(source: string) {
  const directory: string = await mkdtemp(join(tmpdir(), "nuc-arc-"));

  try {
    const file: string = join(directory, "program.nuc");
    await writeFile(file, source, "utf8");
    const stdout: string = await execute(["--json", file]);

    try {
      const result: unknown = JSON.parse(stdout);
      return result;
    } catch (error) {
      throw new Error("Nucleoid returned invalid JSON", { cause: error });
    }
  } finally {
    await rm(directory, { recursive: true });
  }
}

export function snapshot(value: unknown): {
  [property: string]: unknown;
} {
  if (!validateSnapshot(value)) {
    throw new Error(
      `Nucleoid returned invalid object state: ${JSON.stringify(
        validateSnapshot.errors
      )}`
    );
  }
  return value;
}
