import { EngineQuickStartPage } from "../components/engine-pages";

export const meta = () => [{ title: "Google Ads | agent-ads" }];

export default function GoogleEngineRoute() {
  return <EngineQuickStartPage engineId="google" />;
}
