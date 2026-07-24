import { execFileSync } from "node:child_process";
import { statSync } from "node:fs";
import { resolve } from "node:path";

const websiteRoot = resolve(import.meta.dirname, "../..");

export function lastUpdatedIso(relativePath: string) {
  const filePath = resolve(websiteRoot, relativePath);
  try {
    const committed = execFileSync("git", ["log", "-1", "--format=%cI", "--", filePath], {
      cwd: websiteRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    if (committed) return committed;
  } catch {
    // Uncommitted or outside git history.
  }
  try {
    return statSync(filePath).mtime.toISOString();
  } catch {
    return undefined;
  }
}

export function formatLastUpdatedZh(value: string | undefined) {
  if (!value) return "";
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) return "";
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "long",
    day: "numeric",
  }).format(date);
}
