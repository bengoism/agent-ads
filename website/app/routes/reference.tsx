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
      "Full provider reference for Meta, Google Ads, TikTok, Pinterest, LinkedIn, and X, with routing guides and workflow docs.",
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
              Full provider reference
              <span className="text-fg/78"> for every surface.</span>
            </>
          }
          lede="Browse the provider guides for Meta, Google Ads, TikTok, Pinterest, LinkedIn, and X when you need exact flags, auth requirements, or deeper workflow examples."
          detail={`${engineOrder.reduce((c, id) => c + generatedContent.references[id].length, 0)} guides`}
          actions={
            <>
              <Link className={btnPrimary} to="/">
                Quick Start
              </Link>
              <Link className={btnSecondary} to="/skills">
                Skill guide
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
          eyebrow="Providers"
          title="Open a provider guide"
          copy="Each provider page groups the commands, auth details, and workflow docs that already exist for that surface."
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
                cta="Open provider docs"
                tags={engine.tags}
              />
            );
          })}
        </div>
      </section>
    </>
  );
}
