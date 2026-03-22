import { Link } from "react-router";
import { btnPrimary, btnSecondary, eyebrowCls } from "../components/docs";

export const meta = () => [{ title: "Not found | agent-ads" }];

export default function NotFoundRoute() {
  return (
    <section
      className="grid gap-4 p-[1.2rem] rounded bg-gradient-to-b from-[rgba(28,27,28,0.96)] to-[rgba(16,16,17,0.98)] border-[0.5px] border-outline shadow-ambient reveal"
      data-reveal
    >
      <span className={eyebrowCls}>404</span>
      <h1 className="m-0 text-[clamp(2rem,4vw,3rem)] tracking-[-0.04em]">Route not found</h1>
      <p className="m-0 text-fg-muted leading-[1.72]">There isn&apos;t a page at this path.</p>
      <div className="flex flex-col items-stretch gap-3 sm:flex-row sm:items-center">
        <Link className={btnPrimary} to="/">
          Back to overview
        </Link>
        <Link className={btnSecondary} to="/reference">
          Open reference hub
        </Link>
      </div>
    </section>
  );
}
