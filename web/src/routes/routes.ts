import { lazy } from "solid-js";

export const routes = {
  path: "/",
  component: lazy(() => import("./layout")),
  children: [
    { path: "/", component: lazy(() => import("./index")) },
    { path: "/charts", component: lazy(() => import("./charts/index")) },
    { path: "/charts/review", component: lazy(() => import("./charts/review")) },
    {
      path: "/tournaments",
      component: lazy(() => import("./tournaments/layout")),
      children: [
        { path: "/", component: lazy(() => import("./tournaments/index")) },
        {
          path: "/:tournament",
          component: lazy(() => import("./tournaments/[tournament]/layout")),
          children: [
            { path: "/", component: lazy(() => import("./tournaments/[tournament]/index")) },
            { path: "/charts", component: lazy(() => import("./tournaments/[tournament]/charts/index")) },
            { path: "/results", component: lazy(() => import("./tournaments/[tournament]/results/index")) },
            {
              path: "/leaderboard/individual",
              component: lazy(() => import("./tournaments/[tournament]/leaderboard-individual")),
            },
            { path: "/leaderboard/team", component: lazy(() => import("./tournaments/[tournament]/leaderboard-team")) },
            { path: "/admin", component: lazy(() => import("./tournaments/[tournament]/admin/index")) },
          ],
        },
      ],
    },
    {
      path: "/account",
      component: lazy(() => import("./account/layout")),
      children: [
        { path: "/", component: lazy(() => import("./account/index")) },
        { path: "/login", component: lazy(() => import("./account/login/index")) },
        { path: "/register", component: lazy(() => import("./account/register/index")) },
        { path: "/forgot", component: lazy(() => import("./account/forgot/index")) },
        { path: "/reset", component: lazy(() => import("./account/reset/index")) },
        { path: "/oauth", component: lazy(() => import("./account/oauth/index")) },
        { path: "/verify", component: lazy(() => import("./account/verify/index")) },
        {
          path: "/settings",
          component: lazy(() => import("./account/settings/layout")),
          children: [
            { path: "/", component: lazy(() => import("./account/settings/index")) },
            { path: "/info", component: lazy(() => import("./account/settings/info/index")) },
            { path: "/password", component: lazy(() => import("./account/settings/password/index")) },
            { path: "/oauth", component: lazy(() => import("./account/settings/oauth/index")) },
            { path: "/phira", component: lazy(() => import("./account/settings/phira/index")) },
          ],
        },
      ],
    },
    {
      path: "/wiki",
      component: lazy(() => import("./wiki/layout")),
      children: [
        { path: "/", component: lazy(() => import("./wiki/index")) },
        { path: "/create", component: lazy(() => import("./wiki/create/index")) },
        { path: "/:article", component: lazy(() => import("./wiki/[article]/index")) },
      ],
    },
    {
      path: "/bulletin",
      component: lazy(() => import("./bulletin/layout")),
      children: [
        { path: "/", component: lazy(() => import("./bulletin/index")) },
        { path: "/create", component: lazy(() => import("./bulletin/create")) },
        { path: "/:article", component: lazy(() => import("./bulletin/[article]/index")) },
      ],
    },
    {
      path: "/admin",
      component: lazy(() => import("./admin/layout")),
      children: [
        { path: "/", component: lazy(() => import("./admin/index")) },
        { path: "/users", component: lazy(() => import("./admin/users/index")) },
        { path: "/statistics", component: lazy(() => import("./admin/statistics/index")) },
        { path: "/captcha", component: lazy(() => import("./admin/captcha/index")) },
        { path: "/email", component: lazy(() => import("./admin/email/index")) },
        { path: "/edit", component: lazy(() => import("./admin/edit/index")) },
        { path: "/phira", component: lazy(() => import("./admin/phira/index")) },
        { path: "/logs", component: lazy(() => import("./admin/logs/index")) },
        { path: "/media", component: lazy(() => import("./admin/media/index")) },
        { path: "/oauth", component: lazy(() => import("./admin/oauth/index")) },
        { path: "/tournaments", component: lazy(() => import("./admin/tournaments/index")) },
      ],
    },
    {
      path: "/users",
      component: lazy(() => import("./users/layout")),
      children: [
        { path: "/", component: lazy(() => import("./users/index")) },
        { path: "/:user", component: lazy(() => import("./users/[user]/index")) },
      ],
    },
    {
      path: "/error",
      component: lazy(() => import("./error/layout")),
      children: [
        { path: "/401", component: lazy(() => import("./error/e401")) },
        { path: "/403", component: lazy(() => import("./error/e403")) },
        { path: "/404", component: lazy(() => import("./error/e404")) },
        { path: "/412", component: lazy(() => import("./error/e412")) },
        { path: "/418", component: lazy(() => import("./error/e418")) },
        { path: "/500", component: lazy(() => import("./error/e500")) },
        { path: "/502", component: lazy(() => import("./error/e502")) },
        { path: "/unknown", component: lazy(() => import("./error/unknown")) },
      ],
    },
    { path: "*", component: lazy(() => import("./error/e404")) },
  ],
};
