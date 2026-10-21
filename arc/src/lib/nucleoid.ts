import { execFile } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

export async function run(source) {
  const directory = await mkdtemp(join(tmpdir(), "nuc-arc-"));

  try {
    const file = join(directory, "program.nuc");
    await writeFile(file, source, "utf8");
    const executable =
      process.env.NUCLEOID_EXECUTABLE ??
      String.raw`C:\Users\canmi\dev\Nucleoid\target\debug\nucleoid.exe`;
    const stdout = await new Promise((resolve, reject) => {
      execFile(
        executable,
        ["--json", file],
        { encoding: "utf8", windowsHide: true },
        (error, stdout, stderr) => {
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

    if (typeof stdout !== "string") {
      throw new Error("Nucleoid returned non-text output");
    }

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
