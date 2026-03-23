import {
  isRouteErrorResponse,
  Links,
  Meta,
  NavLink,
  Outlet,
  Scripts,
  ScrollRestoration,
  useLocation,
  useRouteError,
} from "react-router";
import { useEffect, useState, type ReactNode } from "react";
import { Analytics } from "@vercel/analytics/react";
import appStylesHref from "./app.css?url";
import { headerLinks, repoLinks, sidebarNav } from "./content/site";
import { btnPrimary, eyebrowCls, useRevealObserver } from "./components/docs";
import { TocProvider, useTocItems } from "./components/toc-context";

export const links = () => [
  { rel: "preconnect", href: "https://fonts.googleapis.com" },
  { rel: "preconnect", href: "https://fonts.gstatic.com", crossOrigin: "anonymous" },
  {
    rel: "stylesheet",
    href:
      "https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;700&family=Playfair+Display:ital,wght@1,400;1,700&display=swap",
  },
  { rel: "stylesheet", href: appStylesHref },
];

export function Layout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body>
        {children}
        <Analytics />
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

function useActiveId(ids: string[]) {
  const [activeId, setActiveId] = useState("");
  useEffect(() => {
    if (!ids.length) return;
    const elements = ids
      .map((id) => document.getElementById(id))
      .filter(Boolean) as HTMLElement[];
    if (!elements.length) return;

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id);
          }
        }
      },
      { rootMargin: "0px 0px -65% 0px", threshold: 0 },
    );
    for (const el of elements) observer.observe(el);
    return () => observer.disconnect();
  }, [ids]);
  return activeId;
}

function RightToc() {
  const items = useTocItems();
  const activeId = useActiveId(items.map((i) => i.id));
  if (!items.length) return null;
  return (
    <aside
      className="hidden lg:block sticky top-14 h-[calc(100vh-3.5rem)] overflow-y-auto py-5 pr-3"
      aria-label="On this page"
    >
      <nav className="grid gap-[0.15rem]">
        <div className="text-[0.68rem] font-bold tracking-[0.1em] uppercase text-fg-dim py-2 px-3 pb-[0.15rem]">
          On this page
        </div>
        {items.map((item) => (
          <a
            key={item.id}
            href={`#${item.id}`}
            className={`block py-[0.35rem] px-3 rounded text-[0.82rem] transition-[background,color] duration-150 motion-reduce:transition-none truncate ${
              activeId === item.id
                ? "text-accent bg-accent/8"
                : "text-fg-muted hover:text-fg hover:bg-surface-highest/35"
            }`}
          >
            {item.label}
          </a>
        ))}
      </nav>
    </aside>
  );
}

export default function App() {
  const location = useLocation();
  useRevealObserver(location.pathname);

  return (
    <TocProvider>
    <div className="relative">
      {/* ── Topbar ─────────────────────────────────────────── */}
      <header className="fixed inset-x-0 top-0 z-30 flex items-center justify-between h-14 px-4 md:px-[1.35rem] bg-surface/92 backdrop-blur-[20px] border-b-[0.5px] border-outline">
        <NavLink
          className="inline-flex items-center gap-2 text-[0.95rem] font-extrabold tracking-[-0.04em] text-fg"
          to="/"
        >
          agent-ads
        </NavLink>

        <nav
          className="flex items-center gap-5 text-[0.88rem] text-fg-muted"
          aria-label="External links"
        >
          {headerLinks.map((link) => (
            <a
              key={link.href}
              href={link.href}
              target="_blank"
              rel="noreferrer"
              className="hover:text-fg"
            >
              {link.label}
            </a>
          ))}
        </nav>
      </header>

      {/* ── App shell ──────────────────────────────────────── */}
      <div className="grid grid-cols-1 md:grid-cols-[15rem_minmax(0,1fr)] lg:grid-cols-[15rem_minmax(0,1fr)_14rem] max-w-[1440px] mx-auto pt-14 min-h-screen">
        {/* ── Sidebar ────────────────────────────────────── */}
        <aside
          className="hidden md:block sticky top-14 h-[calc(100vh-3.5rem)] overflow-y-auto border-r-[0.5px] border-outline py-5"
          aria-label="Documentation navigation"
        >
          <nav className="grid gap-6 px-3">
            <div className="grid gap-[0.15rem]">
              <div className="text-[0.68rem] font-bold tracking-[0.1em] uppercase text-fg-dim py-2 px-3 pb-[0.15rem]">
                {sidebarNav.navigation.label}
              </div>
              <div className="text-[0.82rem] font-semibold text-fg px-3 pb-2">
                {sidebarNav.navigation.subtitle}
              </div>
              {sidebarNav.navigation.items.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  end={"end" in item ? item.end : false}
                  className={({ isActive }) =>
                    `block py-2 px-3 rounded text-[0.88rem] font-medium transition-[background,color] duration-150 motion-reduce:transition-none ${
                      isActive
                        ? "text-accent bg-accent/8"
                        : "text-fg-muted hover:text-fg hover:bg-surface-highest/35"
                    }`
                  }
                >
                  {item.label}
                </NavLink>
              ))}
            </div>

            <div className="grid gap-[0.15rem]">
              <div className="text-[0.68rem] font-bold tracking-[0.1em] uppercase text-fg-dim py-2 px-3 pb-[0.15rem]">
                {sidebarNav.engines.label}
              </div>
              {sidebarNav.engines.items.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) =>
                    `block py-2 px-3 rounded text-[0.88rem] font-medium transition-[background,color] duration-150 motion-reduce:transition-none ${
                      isActive
                        ? "text-accent bg-accent/8"
                        : "text-fg-muted hover:text-fg hover:bg-surface-highest/35"
                    }`
                  }
                >
                  {item.label}
                </NavLink>
              ))}
            </div>
          </nav>
        </aside>

        {/* ── Main content ───────────────────────────────── */}
        <main className="grid gap-5 min-w-0 px-3 pt-4 pb-8 sm:px-4 sm:pt-6 sm:pb-12 md:px-12 md:pt-8 md:pb-16">
          {/* Mobile nav */}
          <div className="sticky top-[calc(3.5rem+0.5rem)] z-[12] flex flex-wrap gap-2 p-3 rounded bg-surface-container/78 backdrop-blur-[16px] border-[0.5px] border-outline md:hidden">
            <nav className="flex flex-wrap gap-2" aria-label="Mobile navigation">
              {sidebarNav.navigation.items.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  end={"end" in item ? item.end : false}
                  className={({ isActive }) =>
                    `inline-flex items-center min-h-8 py-[0.4rem] px-[0.65rem] rounded text-[0.82rem] font-semibold ${
                      isActive
                        ? "bg-accent/12 text-accent"
                        : "bg-surface-highest/55 text-fg-muted"
                    }`
                  }
                >
                  {item.label}
                </NavLink>
              ))}
              {sidebarNav.engines.items.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) =>
                    `inline-flex items-center min-h-8 py-[0.4rem] px-[0.65rem] rounded text-[0.82rem] font-semibold ${
                      isActive
                        ? "bg-accent/12 text-accent"
                        : "bg-surface-highest/55 text-fg-muted"
                    }`
                  }
                >
                  {item.label}
                </NavLink>
              ))}
            </nav>
          </div>

          <div className="grid gap-[clamp(4rem,7vw,5.75rem)] min-w-0 max-w-[52rem]">
            <Outlet />
          </div>

          <footer className="flex flex-col items-start gap-3 sm:flex-row sm:items-center sm:justify-between py-6 mt-8 border-t-[0.5px] border-outline text-[0.78rem] text-fg-dim uppercase tracking-[0.06em]">
            <div className="flex items-center gap-6">
              <a href={repoLinks.github} target="_blank" rel="noreferrer" className="hover:text-fg-muted">
                X / Twitter
              </a>
              <a href={repoLinks.github} target="_blank" rel="noreferrer" className="hover:text-fg-muted">
                Changelog
              </a>
              <a href={repoLinks.github} target="_blank" rel="noreferrer" className="hover:text-fg-muted">
                Docs Repository
              </a>
            </div>
          </footer>
        </main>

        <RightToc />
      </div>
    </div>
    </TocProvider>
  );
}

export function HydrateFallback() {
  return <div className="grid place-items-center min-h-screen p-8">Loading agent-ads docs...</div>;
}

export function ErrorBoundary() {
  const error = useRouteError();
  const title = isRouteErrorResponse(error) ? `${error.status} ${error.statusText}` : "Render error";
  const description = isRouteErrorResponse(error)
    ? "The requested route could not be rendered."
    : error instanceof Error
      ? error.message
      : "Unknown error";

  return (
    <div className="grid place-items-center min-h-screen p-8">
      <section className="grid gap-4 max-w-[38rem] p-[1.4rem] rounded bg-gradient-to-b from-[rgba(28,27,28,0.96)] to-[rgba(16,16,17,0.98)] border-[0.5px] border-outline shadow-ambient">
        <span className={eyebrowCls}>Site error</span>
        <h1 className="m-0 text-[clamp(2rem,4vw,3rem)] tracking-[-0.04em]">{title}</h1>
        <p className="m-0 text-fg-muted leading-[1.72]">{description}</p>
        <NavLink className={btnPrimary} to="/">
          Return to docs
        </NavLink>
      </section>
    </div>
  );
}
