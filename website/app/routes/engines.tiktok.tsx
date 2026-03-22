import { EngineQuickStartPage } from "../components/engine-pages";

export const meta = () => [{ title: "TikTok | agent-ads" }];

export default function TikTokEngineRoute() {
  return <EngineQuickStartPage engineId="tiktok" />;
}
