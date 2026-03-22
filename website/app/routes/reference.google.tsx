import { EngineReferencePage } from "../components/engine-pages";

export const meta = () => [{ title: "Google Ads reference | agent-ads" }];

export default function GoogleReferenceRoute() {
  return <EngineReferencePage engineId="google" />;
}
