import { execFile } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

export async function run(source: string): Promise<unknown> {
  const directory = await mkdtemp(join(tmpdir(), "nuc-arc-"));

  try {
    const file = join(directory, "program.nuc");
    await writeFile(file, source, "utf8");
    const executable =
      process.env.NUCLEOID_EXECUTABLE ??
      String.raw`C:\Users\canmi\dev\Nucleoid\target\debug\nucleoid.exe`;
    const stdout = await new Promise<string>((resolve, reject) => {
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

    try {
      const result: unknown = JSON.parse(stdout);
      return result;
    } catch (error: unknown) {
      throw new Error("Nucleoid returned invalid JSON", { cause: error });
    }
  } finally {
    await rm(directory, { recursive: true });
  }
}
