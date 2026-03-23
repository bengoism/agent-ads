import { CommandPanel, SectionHeader } from "../components/docs";

export const meta = () => [
  { title: "Auth | agent-ads" },
  { name: "description", content: "Authentication setup for Meta, Google Ads, TikTok, Pinterest, LinkedIn, and X." },
];

export default function AuthRoute() {
  return (
    <>
      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Authentication"
          title="Auth"
          copy="Each provider has its own credential model. The CLI resolves secrets with a fixed precedence: shell env > OS credential store. Secrets never come from flags or config files."
        />

        <CommandPanel
          eyebrow="Cross-provider status"
          title="Check all configured providers"
          command="agent-ads auth status"
          copyKey="auth-status"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Meta"
          title="Meta Authentication"
          copy="Meta requires an access token with ads_read permission. Optionally add business_management to discover businesses and ad accounts."
        />
        <CommandPanel
          compact
          eyebrow="Store token"
          title="Persist to OS credential store"
          command="agent-ads meta auth set"
          copyKey="meta-auth-set"
        />
        <CommandPanel
          compact
          eyebrow="Environment override"
          title="One-off session token"
          command="export META_ADS_ACCESS_TOKEN=EAABs..."
          copyKey="meta-auth-env"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Google Ads"
          title="Google Ads Authentication"
          copy="Google Ads requires a developer token, OAuth client ID/secret, and a refresh token. All four values go into the OS credential store."
        />
        <CommandPanel
          compact
          eyebrow="Store credentials"
          title="Guided auth setup"
          command="agent-ads google auth set"
          copyKey="google-auth-set"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="TikTok"
          title="TikTok Authentication"
          copy="TikTok uses app credentials (app ID + secret) to obtain a 24-hour access token. The CLI handles token refresh."
        />
        <CommandPanel
          compact
          eyebrow="Store credentials"
          title="Guided auth setup"
          command="agent-ads tiktok auth set"
          copyKey="tiktok-auth-set"
        />
        <CommandPanel
          compact
          eyebrow="Refresh"
          title="Refresh an expired token"
          command="agent-ads tiktok auth refresh"
          copyKey="tiktok-auth-refresh"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="Pinterest"
          title="Pinterest Authentication"
          copy="Pinterest uses OAuth with app ID, app secret, and a refresh token. Tokens are stored in the OS credential store."
        />
        <CommandPanel
          compact
          eyebrow="Store credentials"
          title="Guided auth setup"
          command="agent-ads pinterest auth set"
          copyKey="pinterest-auth-set"
        />
        <CommandPanel
          compact
          eyebrow="Refresh"
          title="Refresh an expired token"
          command="agent-ads pinterest auth refresh"
          copyKey="pinterest-auth-refresh"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="X"
          title="X Authentication"
          copy="X uses four OAuth 1.0a secrets: consumer key, consumer secret, access token, and access token secret."
        />
        <CommandPanel
          compact
          eyebrow="Store credentials"
          title="Guided auth setup"
          command="agent-ads x auth set"
          copyKey="x-auth-set"
        />
      </section>

      <section className="grid gap-6">
        <SectionHeader
          eyebrow="LinkedIn"
          title="LinkedIn Authentication"
          copy="LinkedIn uses an access token only. Store it in the OS credential store or override it from the shell for one-off sessions."
        />
        <CommandPanel
          compact
          eyebrow="Store token"
          title="Guided auth setup"
          command="agent-ads linkedin auth set"
          copyKey="linkedin-auth-set"
        />
        <CommandPanel
          compact
          eyebrow="Environment override"
          title="One-off session token"
          command="export LINKEDIN_ADS_ACCESS_TOKEN=access-token"
          copyKey="linkedin-auth-env"
        />
      </section>
    </>
  );
}
