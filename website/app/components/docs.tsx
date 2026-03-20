import { useEffect, useState, type ReactNode } from "react";
import ReactMarkdown from "react-markdown";
import { Link } from "react-router";
import remarkGfm from "remark-gfm";

/* ─── Shared class-string constants ─────────────────────── */

const btnBase =
  "inline-flex items-center justify-center min-h-[2.75rem] py-[0.65rem] px-4 rounded text-[0.9rem] font-bold tracking-[-0.01em] transition-[transform,background,color] duration-[180ms] hover:-translate-y-px motion-reduce:transition-none motion-reduce:transform-none w-full sm:w-auto";

export const btnPrimary = `${btnBase} bg-gradient-to-br from-accent to-accent-strong text-[#0d0f19]`;
export const btnSecondary = `${btnBase} border-[0.5px] border-[rgba(70,69,84,0.4)] text-fg-muted bg-[rgba(14,14,15,0.48)]`;
export const eyebrowCls =
  "inline-flex text-[0.72rem] font-bold tracking-[0.08em] uppercase text-fg-dim";

/* ─── Utilities ─────────────────────────────────────────── */

async function copyToClipboard(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "");
  textarea.style.position = "absolute";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();
  document.execCommand("copy");
  document.body.removeChild(textarea);
}

export function useRevealObserver(routeKey: string) {
  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    const revealElements = Array.from(document.querySelectorAll<HTMLElement>("[data-reveal]"));

    const revealObserver = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            entry.target.classList.add("is-visible");
            revealObserver.unobserve(entry.target);
          }
        }
      },
      {
        rootMargin: "0px 0px -12% 0px",
        threshold: 0.18,
      },
    );

    for (const revealElement of revealElements) {
      revealElement.classList.remove("is-visible");
      revealObserver.observe(revealElement);
    }

    return () => revealObserver.disconnect();
  }, [routeKey]);
}

/* ─── Components ────────────────────────────────────────── */

export function CopyButton({ copyKey, text }: { copyKey: string; text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    void copyToClipboard(text).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    });
  };

  return (
    <button
      type="button"
      className={`border-0 rounded py-[0.55rem] px-3 cursor-pointer transition-[background,color] duration-[180ms] motion-reduce:transition-none ${
        copied
          ? "bg-accent/14 text-accent"
          : "bg-surface-highest/58 text-fg-muted hover:bg-accent/14 hover:text-accent"
      }`}
      onClick={handleCopy}
      aria-label={copied ? `Copied ${copyKey}` : `Copy ${copyKey}`}
    >
      {copied ? "Copied" : "Copy"}
    </button>
  );
}

export function PageHero({
  eyebrow,
  title,
  lede,
  detail,
  actions,
  aside,
}: {
  eyebrow: string;
  title: ReactNode;
  lede: string;
  detail?: string;
  actions?: ReactNode;
  aside?: ReactNode;
}) {
  return (
    <section
      className="relative grid gap-[clamp(1.75rem,3vw,3rem)] p-[clamp(1.4rem,2vw,2.2rem)] rounded bg-gradient-to-b from-[rgba(28,27,28,0.92)] to-[rgba(18,18,19,0.98)] shadow-ambient overflow-hidden lg:grid-cols-[minmax(0,1.25fr)_minmax(18rem,0.92fr)] before:content-[''] before:absolute before:-right-[10%] before:-bottom-[30%] before:w-[28rem] before:h-[28rem] before:bg-[radial-gradient(circle,rgba(192,193,255,0.18),transparent_66%)] before:pointer-events-none reveal"
      data-reveal
    >
      <div className="grid gap-[1.2rem]">
        <div className="flex flex-wrap gap-2">
          <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-accent/12 text-accent text-[0.72rem] font-bold tracking-[0.06em] uppercase">
            {eyebrow}
          </span>
          {detail ? (
            <span className="inline-flex items-center py-[0.35rem] px-[0.65rem] rounded bg-surface-highest/52 text-fg-muted text-[0.72rem] font-bold tracking-[0.06em] uppercase">
              {detail}
            </span>
          ) : null}
        </div>
        <h1 className="m-0 max-w-[12ch] text-[clamp(2rem,4vw,3rem)] leading-[0.96] tracking-[-0.05em]">
          {title}
        </h1>
        <p className="m-0 text-fg-muted leading-[1.72]">{lede}</p>
        {actions ? (
          <div className="flex flex-col items-stretch gap-3 sm:flex-row sm:items-center">
            {actions}
          </div>
        ) : null}
      </div>
      {aside ? <div className="grid gap-[1.2rem]">{aside}</div> : null}
    </section>
  );
}

export function SectionHeader({
  eyebrow,
  title,
  copy,
}: {
  eyebrow: string;
  title: string;
  copy: string;
}) {
  return (
    <header className="grid gap-[0.4rem] max-w-3xl">
      <span className={eyebrowCls}>{eyebrow}</span>
      <h2 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em]">
        {title}
      </h2>
      <p className="m-0 text-fg-muted leading-[1.72]">{copy}</p>
    </header>
  );
}

export function CommandPanel({
  eyebrow,
  title,
  command,
  copyKey,
  compact = false,
}: {
  eyebrow: string;
  title: string;
  command: string;
  copyKey: string;
  compact?: boolean;
}) {
  return (
    <div
      className={`grid gap-[0.9rem] rounded bg-gradient-to-b from-[rgba(14,14,15,0.96)] to-[rgba(20,20,21,0.96)] border-[0.5px] border-outline shadow-ambient ${
        compact ? "p-[0.9rem]" : "p-4"
      }`}
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <span className={eyebrowCls}>{eyebrow}</span>
          <div className="mt-1 text-fg text-[0.95rem] font-semibold leading-[1.5]">{title}</div>
        </div>
        <CopyButton copyKey={copyKey} text={command} />
      </div>
      <pre className="m-0 max-w-full overflow-x-auto p-4 rounded bg-[rgba(10,10,11,0.95)] border-[0.5px] border-outline text-accent text-[0.88rem] leading-[1.6]">
        <code>{command}</code>
      </pre>
    </div>
  );
}

export function MetricGrid({
  metrics,
}: {
  metrics: Array<{ label: string; value: string }>;
}) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-[0.6rem]">
      {metrics.map((metric) => (
        <div
          key={metric.label}
          className="grid gap-[0.55rem] p-[0.9rem] rounded bg-gradient-to-b from-[rgba(14,14,15,0.88)] to-[rgba(20,20,21,0.94)] shadow-ambient"
        >
          <span className={eyebrowCls}>{metric.label}</span>
          <strong className="text-[0.94rem] leading-[1.5] font-bold">{metric.value}</strong>
        </div>
      ))}
    </div>
  );
}

export function BadgeList({ values }: { values: string[] }) {
  return (
    <div className="flex flex-wrap gap-[0.4rem]">
      {values.map((value) => (
        <span
          key={value}
          className="inline-flex items-center min-h-[1.6rem] py-[0.2rem] px-[0.45rem] rounded bg-surface-highest/62 text-fg-muted text-[0.75rem] font-semibold"
        >
          {value}
        </span>
      ))}
    </div>
  );
}

export function MarkdownBlock({
  markdown,
  className,
  resolveHref,
}: {
  markdown: string;
  className?: string;
  resolveHref?: (href: string) => string;
}) {
  return (
    <div className={`markdown${className ? ` ${className}` : ""}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a: ({ href, children }) => {
            const resolvedHref = href ? resolveHref?.(href) ?? href : href;

            if (!resolvedHref) {
              return <>{children}</>;
            }

            const hrefValue = resolvedHref;
            const isExternal =
              hrefValue.startsWith("http://") ||
              hrefValue.startsWith("https://") ||
              hrefValue.startsWith("mailto:");

            if (isExternal) {
              return (
                <a href={hrefValue} target="_blank" rel="noreferrer">
                  {children}
                </a>
              );
            }

            if (hrefValue.startsWith("#")) {
              return <a href={hrefValue}>{children}</a>;
            }

            return <Link to={hrefValue}>{children}</Link>;
          },
          code: ({ className: codeClassName, children, ...props }) => {
            const isInline = !codeClassName;

            if (isInline) {
              return (
                <code className="markdown__inline-code" {...props}>
                  {children}
                </code>
              );
            }

            return (
              <code className={codeClassName} {...props}>
                {children}
              </code>
            );
          },
          table: ({ children }) => (
            <div className="markdown__table-wrap">
              <table>{children}</table>
            </div>
          ),
        }}
      >
        {markdown}
      </ReactMarkdown>
    </div>
  );
}

export function RouteCard({
  eyebrow,
  title,
  copy,
  to,
  cta,
  tags,
}: {
  eyebrow: string;
  title: string;
  copy: string;
  to: string;
  cta: string;
  tags?: string[];
}) {
  return (
    <article
      className="relative overflow-hidden grid gap-[0.9rem] p-[1.2rem] rounded bg-surface-low border-[0.5px] border-outline shadow-ambient reveal"
      data-reveal
    >
      <span className={eyebrowCls}>{eyebrow}</span>
      <h3 className="m-0 text-[clamp(1.4rem,2.2vw,2rem)] leading-[1.08] tracking-[-0.04em]">
        {title}
      </h3>
      <p className="m-0 text-fg-muted leading-[1.72]">{copy}</p>
      {tags?.length ? <BadgeList values={tags} /> : null}
      <Link
        className="inline-flex items-center text-accent font-bold hover:text-fg"
        to={to}
      >
        {cta}
      </Link>
    </article>
  );
}

export function TerminalBlock({
  filename,
  headerComment,
  lines,
  comment,
}: {
  filename: string;
  headerComment?: string;
  lines: Array<{ num: string; text: string; highlight?: string[] }>;
  comment?: string;
}) {
  function highlightText(text: string, highlights?: string[]) {
    if (!highlights?.length) {
      return <span className="text-fg-muted">{text}</span>;
    }

    const parts: ReactNode[] = [];
    let remaining = text;
    let keyIndex = 0;

    for (const hl of highlights) {
      const idx = remaining.indexOf(hl);
      if (idx === -1) continue;
      if (idx > 0) {
        parts.push(remaining.slice(0, idx));
      }
      parts.push(
        <span key={keyIndex++} className="text-green font-bold">
          {hl}
        </span>,
      );
      remaining = remaining.slice(idx + hl.length);
    }

    if (remaining) {
      parts.push(remaining);
    }

    return <span className="text-fg-muted">{parts}</span>;
  }

  return (
    <div className="rounded-lg overflow-hidden bg-surface-lowest border-[0.5px] border-outline">
      <div className="flex items-center justify-between py-[0.65rem] px-[0.85rem] bg-surface-low border-b-[0.5px] border-outline">
        <div className="flex gap-[0.35rem]">
          <span className="w-[0.55rem] h-[0.55rem] rounded-full bg-[#ff5f57]" />
          <span className="w-[0.55rem] h-[0.55rem] rounded-full bg-[#febc2e]" />
          <span className="w-[0.55rem] h-[0.55rem] rounded-full bg-[#28c840]" />
        </div>
        <span className="text-[0.78rem] text-fg-dim">{filename}</span>
        <div className="flex gap-2">
          <CopyButton copyKey={filename} text={lines.map((l) => l.text).join("\n")} />
        </div>
      </div>
      {headerComment ? (
        <span className="block px-4 pb-1 text-fg-dim font-mono text-[0.8rem]">
          {headerComment}
        </span>
      ) : null}
      <div className="p-4">
        {lines.map((line) => (
          <div key={line.num} className="flex gap-3 leading-[1.7] text-[0.84rem]">
            <span className="text-fg-dim select-none min-w-6 text-right">{line.num}</span>
            {highlightText(line.text, line.highlight)}
          </div>
        ))}
      </div>
      {comment ? (
        <span className="block mt-[0.35rem] px-4 pb-3 text-fg-dim font-mono text-[0.8rem]">
          {comment}
        </span>
      ) : null}
    </div>
  );
}
