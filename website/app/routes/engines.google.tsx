import { EnginePage } from "../components/engine-pages";

export const meta = () => [{ title: "Google Ads | agent-ads" }];

export default function GoogleEngineRoute() {
  return <EnginePage engineId="google" />;
}
