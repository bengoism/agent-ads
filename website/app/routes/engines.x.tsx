import { EnginePage } from "../components/engine-pages";

export const meta = () => [{ title: "X Ads | agent-ads" }];

export default function XEngineRoute() {
  return <EnginePage engineId="x" />;
}
