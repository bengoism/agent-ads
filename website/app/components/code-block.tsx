import { useState } from "react";
import { highlight } from "sugar-high";
import { tokenizeShell, type ShellTokenType } from "./tokenize-shell";

const SHELL_LANGS = new Set(["bash", "shell", "sh", "zsh"]);

const TOKEN_CLASSES: Record<ShellTokenType, string> = {
  command: "text-syn-blue font-bold",
  subcommand: "text-syn-fg",
  flag: "text-syn-cyan",
  string: "text-syn-green",
  value: "text-syn-orange",
  pipe: "text-syn-comment",
  comment: "text-syn-comment italic",
  "env-var": "text-syn-yellow",
  keyword: "text-syn-purple",
  plain: "",
};

function ShellHighlighted({ code }: { code: string }) {
  const lines = tokenizeShell(code);
  return (
    <>
      {lines.map((tokens, i) => (
        <span key={i}>
          {i > 0 ? "\n" : ""}
          {tokens.map((token, j) => (
            <span key={j} className={TOKEN_CLASSES[token.type]}>
              {token.text}
            </span>
          ))}
        </span>
      ))}
    </>
  );
}

function stripPromptPrefix(code: string): string {
  return code
    .split("\n")
    .map((line) => line.replace(/^\s*[$#]\s/, ""))
    .join("\n");
}

export function CodeBlock({
  code,
  language,
  filename,
  showHeader = true,
  showLineNumbers = false,
  highlightLines,
  copyable = true,
}: {
  code: string;
  language?: string;
  filename?: string;
  showHeader?: boolean;
  showLineNumbers?: boolean;
  highlightLines?: number[];
  copyable?: boolean;
}) {
  const [copied, setCopied] = useState(false);
  const isShell = language != null && SHELL_LANGS.has(language);
  const label = filename ?? language;
  const highlightSet = highlightLines ? new Set(highlightLines) : null;

  const handleCopy = () => {
    const textToCopy = isShell ? stripPromptPrefix(code) : code;
    void navigator.clipboard.writeText(textToCopy).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    });
  };

  const codeLines = code.split("\n");

  let codeContent: React.ReactNode;
  if (isShell) {
    codeContent = <ShellHighlighted code={code} />;
  } else if (language) {
    const html = highlight(code);
    codeContent = <span dangerouslySetInnerHTML={{ __html: html }} />;
  } else {
    codeContent = <span className="text-syn-fg">{code}</span>;
  }

  // When line numbers or line highlighting are needed, render line-by-line
  const needsLineWrapper = showLineNumbers || highlightSet;

  return (
    <div className="rounded-lg overflow-hidden border-[0.5px] border-outline">
      {showHeader && label ? (
        <div className="flex items-center justify-between px-4 py-2 bg-surface-low border-b-[0.5px] border-outline">
          <span className="text-[0.72rem] font-bold tracking-[0.08em] uppercase text-fg-dim">
            {label}
          </span>
          {copyable ? (
            <button
              type="button"
              className={`border-0 rounded py-[0.35rem] px-2.5 cursor-pointer text-[0.78rem] font-medium transition-[background,color] duration-[180ms] motion-reduce:transition-none ${
                copied
                  ? "bg-accent/14 text-accent"
                  : "bg-surface-highest/58 text-fg-muted hover:bg-accent/14 hover:text-accent"
              }`}
              onClick={handleCopy}
              aria-label={copied ? "Copied" : "Copy code"}
            >
              {copied ? "Copied" : "Copy"}
            </button>
          ) : null}
        </div>
      ) : null}
      <pre className="m-0 overflow-x-auto bg-[rgba(10,10,11,0.95)] text-[0.88rem] leading-[1.6] font-mono">
        {needsLineWrapper ? (
          <code className="block p-4">
            {codeLines.map((line, i) => (
              <div
                key={i}
                className={`flex${highlightSet?.has(i + 1) ? " bg-accent/[0.06] -mx-4 px-4" : ""}`}
              >
                {showLineNumbers ? (
                  <span className="text-fg-dim select-none min-w-8 text-right pr-4 border-r-[0.5px] border-outline mr-4 shrink-0">
                    {i + 1}
                  </span>
                ) : null}
                <span className="flex-1">
                  {isShell ? (
                    <ShellHighlighted code={line} />
                  ) : language ? (
                    <span dangerouslySetInnerHTML={{ __html: highlight(line) }} />
                  ) : (
                    <span className="text-syn-fg">{line}</span>
                  )}
                </span>
              </div>
            ))}
          </code>
        ) : (
          <code className="block p-4">{codeContent}</code>
        )}
      </pre>
    </div>
  );
}
