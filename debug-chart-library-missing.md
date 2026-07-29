# Chart Library Missing

Status: [OPEN]

## Symptom

用户反馈赛事中的谱面库入口或内容消失，尚未确认是数据缺失、接口空响应还是前端入口隐藏。

## Hypotheses

1. `chart_library` 数据仍存在，但前端入口或路由被隐藏。
2. 服务连接到了不同的数据库，旧谱面数据不在当前数据库中。
3. 可见性过滤把已有谱面库记录过滤掉了。
4. 迁移或清理逻辑删除了 `chart_library` 数据。
5. API 请求失败或返回空数据，前端误显示为谱面库不存在。

## Evidence

## Evidence

- 用户确认消失位置是公共谱面库页面，访问结果为 `404`。
- 后端仍注册 `GET /charts/library`，因此数据库表和后端路由没有因本次问题被移除。
- 前端根级路由没有 `/charts`，原有 `/charts` 只存在于赛事路由下的 `/tournaments/:tournament/charts`。
- 公共谱面库页面组件缺失，导致浏览器访问 `/charts` 时进入 404。

## Fix

- 恢复 `web/src/routes/charts/index.tsx`。
- 在根级路由注册 `/charts`。
- 页面通过现有 `GET /charts/library` 加载公共谱面库，并提供搜索和详情展示。

## Verification

- Type diagnostics: passed.
- `corepack pnpm@10.11.0 -C web lint`: passed.
- `corepack pnpm@10.11.0 -C web build:dev`: passed.
- Awaiting browser verification at `/charts`.
