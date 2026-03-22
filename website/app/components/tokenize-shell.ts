export type ShellTokenType =
  | "command"
  | "subcommand"
  | "flag"
  | "string"
  | "value"
  | "pipe"
  | "comment"
  | "env-var"
  | "keyword"
  | "plain";

export interface ShellToken {
  type: ShellTokenType;
  text: string;
}

const PIPE_TOKENS = new Set(["|", ">", ">>", "&&", "||", ";", "2>", "2>>", "&>"]);
const KEYWORDS = new Set(["export", "source", "eval", "exec", "sudo"]);

export function tokenizeShell(code: string): ShellToken[][] {
  return code.split("\n").map(tokenizeLine);
}

function tokenizeLine(line: string): ShellToken[] {
  const tokens: ShellToken[] = [];
  const trimmed = line.trimStart();

  if (trimmed.startsWith("#")) {
    tokens.push({ type: "comment", text: line });
    return tokens;
  }

  const parts = splitShellTokens(line);
  let seenCommand = false;
  let afterFlag = false;

  for (const part of parts) {
    if (/^\s+$/.test(part)) {
      tokens.push({ type: "plain", text: part });
      continue;
    }

    if (part === "\\") {
      tokens.push({ type: "plain", text: part });
      continue;
    }

    if (PIPE_TOKENS.has(part)) {
      tokens.push({ type: "pipe", text: part });
      seenCommand = false;
      afterFlag = false;
      continue;
    }

    // Leading $ or # prompt marker
    if ((part === "$" || part === "#") && tokens.every((t) => t.type === "plain")) {
      tokens.push({ type: "pipe", text: part });
      continue;
    }

    if (part.startsWith("#")) {
      tokens.push({ type: "comment", text: part });
      continue;
    }

    if (KEYWORDS.has(part)) {
      tokens.push({ type: "keyword", text: part });
      continue;
    }

    if (part.startsWith("$") || part.startsWith("${")) {
      tokens.push({ type: "env-var", text: part });
      afterFlag = false;
      continue;
    }

    if (part.startsWith("--") || (part.startsWith("-") && part.length > 1 && !/^-\d/.test(part))) {
      tokens.push({ type: "flag", text: part });
      afterFlag = !part.includes("=");
      continue;
    }

    if (part.startsWith('"') || part.startsWith("'")) {
      tokens.push({ type: "string", text: part });
      afterFlag = false;
      continue;
    }

    if (!seenCommand) {
      tokens.push({ type: "command", text: part });
      seenCommand = true;
      afterFlag = false;
      continue;
    }

    if (afterFlag) {
      tokens.push({ type: "value", text: part });
      afterFlag = false;
      continue;
    }

    tokens.push({ type: "subcommand", text: part });
  }

  return tokens;
}

function splitShellTokens(line: string): string[] {
  const tokens: string[] = [];
  let i = 0;

  while (i < line.length) {
    if (line[i] === " " || line[i] === "\t") {
      let ws = "";
      while (i < line.length && (line[i] === " " || line[i] === "\t")) {
        ws += line[i];
        i++;
      }
      tokens.push(ws);
      continue;
    }

    if (line[i] === '"' || line[i] === "'") {
      const quote = line[i];
      let str = quote;
      i++;
      while (i < line.length && line[i] !== quote) {
        if (line[i] === "\\" && i + 1 < line.length) {
          str += line[i] + line[i + 1];
          i += 2;
        } else {
          str += line[i];
          i++;
        }
      }
      if (i < line.length) {
        str += line[i];
        i++;
      }
      tokens.push(str);
      continue;
    }

    let token = "";
    while (i < line.length && line[i] !== " " && line[i] !== "\t") {
      token += line[i];
      i++;
    }
    tokens.push(token);
  }

  return tokens;
}
