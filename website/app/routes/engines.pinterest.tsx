import { EngineQuickStartPage } from "../components/engine-pages";

export const meta = () => [{ title: "Pinterest | agent-ads" }];

export default function PinterestEngineRoute() {
  return <EngineQuickStartPage engineId="pinterest" />;
}
