import { Link } from "react-router";
import { generatedContent } from "../generated/content";
import { engineOrder, engines } from "../content/site";
import {
  btnPrimary,
  btnSecondary,
  MetricGrid,
  PageHero,
  RouteCard,
  SectionHeader,
} from "../components/docs";

export const meta = () => [
  { title: "Reference | agent-ads" },
  {
    name: "description",
    content:
      "Full command reference for Meta, Google Ads, TikTok, Pinterest, and LinkedIn, with engine guides and workflow docs.",
  },
];

export default function ReferenceRoute() {
  return (
    <>
      <div id="reference-overview">
        <PageHero
          eyebrow="Reference"
          title={
            <>
              Full command reference
              <span className="text-fg/78"> for every engine.</span>
            </>
          }
          lede="Browse the engine guides for Meta, Google Ads, TikTok, Pinterest, and LinkedIn when you need exact flags, auth requirements, or deeper workflow examples."
          detail={`${engineOrder.reduce((c, id) => c + generatedContent.references[id].length, 0)} guides`}
          actions={
            <>
              <Link className={btnPrimary} to="/">
                Quick Start
              </Link>
              <Link className={btnSecondary} to="/skills">
                Agent integration
              </Link>
            </>
          }
          aside={
            <MetricGrid
              metrics={[
                { label: "Engines", value: String(engineOrder.length) },
                {
                  label: "Guides",
                  value: String(
                    engineOrder.reduce(
                      (count, engineId) => count + generatedContent.references[engineId].length,
                      0,
                    ),
                  ),
                },
                { label: "Formats", value: "JSON, JSONL, CSV" },
              ]}
            />
          }
        />
      </div>

      <section id="reference-engines" className="grid gap-6">
        <SectionHeader
          eyebrow="Engines"
          title="Open an engine guide"
          copy="Each engine page groups the commands, auth details, and workflow docs that already exist for that surface."
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {engineOrder.map((engineId) => {
            const engine = engines[engineId];
            return (
              <RouteCard
                key={engineId}
                eyebrow={engine.eyebrow}
                title={engine.name}
                copy={`${generatedContent.references[engineId].length} guides. ${engine.referenceLead}`}
                to={`/engines/${engineId}`}
                cta="Open engine docs"
                tags={engine.tags}
              />
            );
          })}
        </div>
      </section>
    </>
  );
}
