import { CommandPanel, SectionHeader } from "../components/docs";

export const meta = () => [
  { title: "Config | agent-ads" },
  { name: "description", content: "Configuration file, output flags, and format options for agent-ads." },
];

export default function ConfigRoute() {
  return (
    <>
      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Configuration"
          title="Config"
          copy="Non-secret configuration follows a fixed precedence: CLI flags > shell env > agent-ads.config.json. Secrets always use the credential store or shell env."
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Config file"
          title="agent-ads.config.json"
          copy="Place this file in your project root. The CLI looks for it in the current directory and walks up to the filesystem root."
        />
        <CommandPanel
          eyebrow="Inspect config"
          title="Show resolved config path"
          command="agent-ads meta config path"
          copyKey="config-path"
        />
        <CommandPanel
          compact
          eyebrow="Validate"
          title="Check config file syntax"
          command="agent-ads meta config validate"
          copyKey="config-validate"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Output"
          title="Output Flags"
          copy="Shared output flags work the same way across all engines."
        />
        <CommandPanel
          compact
          eyebrow="Format"
          title="JSON (default), JSONL, or CSV"
          command="agent-ads meta insights query --format csv --output report.csv"
          copyKey="output-format"
        />
        <CommandPanel
          compact
          eyebrow="Pretty print"
          title="Human-readable JSON"
          command="agent-ads meta insights query --pretty"
          copyKey="output-pretty"
        />
        <CommandPanel
          compact
          eyebrow="Envelope"
          title="Wrap response with metadata and paging"
          command="agent-ads meta insights query --envelope"
          copyKey="output-envelope"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Doctor"
          title="Doctor Commands"
          copy="Each engine has a doctor command that verifies your setup. Add --api to also test API connectivity."
        />
        <CommandPanel
          compact
          eyebrow="Meta"
          title="Verify Meta setup"
          command="agent-ads meta doctor --api"
          copyKey="doctor-meta"
        />
        <CommandPanel
          compact
          eyebrow="Google"
          title="Verify Google setup"
          command="agent-ads google doctor --api"
          copyKey="doctor-google"
        />
        <CommandPanel
          compact
          eyebrow="TikTok"
          title="Verify TikTok setup"
          command="agent-ads tiktok doctor --api"
          copyKey="doctor-tiktok"
        />
        <CommandPanel
          compact
          eyebrow="Pinterest"
          title="Verify Pinterest setup"
          command="agent-ads pinterest doctor --api"
          copyKey="doctor-pinterest"
        />
        <CommandPanel
          compact
          eyebrow="LinkedIn"
          title="Verify LinkedIn setup"
          command="agent-ads linkedin doctor --api"
          copyKey="doctor-linkedin"
        />
      </section>
    </>
  );
}
