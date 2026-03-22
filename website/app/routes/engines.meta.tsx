import { EngineQuickStartPage } from "../components/engine-pages";

export const meta = () => [{ title: "Meta Ads | agent-ads" }];

export default function MetaEngineRoute() {
  return <EngineQuickStartPage engineId="meta" />;
}
