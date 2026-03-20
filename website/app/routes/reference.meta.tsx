import { EngineReferencePage } from "../components/engine-pages";

export const meta = () => [{ title: "Meta reference | agent-ads" }];

export default function MetaReferenceRoute() {
  return <EngineReferencePage engineId="meta" />;
}
