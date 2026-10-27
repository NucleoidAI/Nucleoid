import { execFile } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

async function execute(args) {
  const executable: string =
    process.env.NUCLEOID_EXECUTABLE ??
    String.raw`C:\Users\canmi\dev\Nucleoid\target\debug\nucleoid.exe`;
  const stdout: string = await new Promise((resolve, reject) => {
    execFile(
      executable,
      args,
      { encoding: "utf8", windowsHide: true },
      (error, stdout: string, stderr: string) => {
        if (error) {
          reject(
            new Error(
              `Nucleoid execution failed: ${stderr.trim() || error.message}`,
              { cause: error }
            )
          );
        } else {
          resolve(stdout);
        }
      }
    );
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
      const result = JSON.parse(stdout);
      return result;
    } catch (error) {
      throw new Error("Nucleoid returned invalid JSON", { cause: error });
    }
  } finally {
    await rm(directory, { recursive: true });
  }
}
