import { index, route, type RouteConfig } from "@react-router/dev/routes";

export default [
  index("routes/home.tsx"),
  route("auth", "routes/auth.tsx"),
  route("config", "routes/config.tsx"),
  route("engines/meta", "routes/engines.meta.tsx"),
  route("engines/google", "routes/engines.google.tsx"),
  route("engines/tiktok", "routes/engines.tiktok.tsx"),
  route("engines/pinterest", "routes/engines.pinterest.tsx"),
  route("reference", "routes/reference.tsx"),
  route("reference/meta", "routes/reference.meta.tsx"),
  route("reference/google", "routes/reference.google.tsx"),
  route("reference/tiktok", "routes/reference.tiktok.tsx"),
  route("reference/pinterest", "routes/reference.pinterest.tsx"),
  route("skills", "routes/skills.tsx"),
  route("*", "routes/not-found.tsx"),
] satisfies RouteConfig;
