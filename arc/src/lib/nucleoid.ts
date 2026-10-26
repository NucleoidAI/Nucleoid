import { execFile } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

async function execute(args) {
  const executable =
    process.env.NUCLEOID_EXECUTABLE ??
    String.raw`C:\Users\canmi\dev\Nucleoid\target\debug\nucleoid.exe`;
  const { stdout } = await execFileAsync(executable, args, {
    encoding: "utf8",
    windowsHide: true,
  }).catch((error) => {
    throw new Error(
      `Nucleoid execution failed: ${error.stderr?.trim() || error.message}`,
      { cause: error }
    );
  });
  return stdout;
}

export async function version() {
  const stdout = await execute(["--version"]);
  const match = /^nucleoid\s+(\S+)\s*$/.exec(stdout);
  if (!match) {
    throw new Error("Nucleoid returned invalid version output");
  }
  return match[1];
}

export async function run(source: string) {
  const directory = await mkdtemp(join(tmpdir(), "nuc-arc-"));

  try {
    const file = join(directory, "program.nuc");
    await writeFile(file, source, "utf8");
    const stdout = await execute(["--json", file]);

    try {
      const result = JSON.parse(stdout);
      return result;
    } catch (error) {
      throw new Error("Nucleoid returned invalid JSON", { cause: error });
    }
  } finally {
    await rm(directory, { recursive: true });
  }
}
