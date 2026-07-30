# 赛事生命周期重设计计划

## Summary

重设计赛事生命周期，使赛事组织者可以在赛事管理界面手动切换未归档赛事的任意阶段，并为报名、比赛、评审、结束四个阶段分别配置“手动”或“定时”模式及阶段开启时间。服务器通过后台任务自动推进定时阶段；赛事进入“已归档”后默认锁定，只有 DevOps 可以修改，且 DevOps 可按赛事开启“允许组织者修改归档赛事”的授权开关。

## Current State Analysis

- 生命周期当前为 `draft → registration → running → review → finished → archived`，后端 `Lifecycle::can_transition_to` 只允许相邻阶段向前推进，前端管理页也只有“推进到下一阶段”操作。
- `PATCH /tournaments/{id}` 已支持更新 `lifecycle` 和四个赛事时间字段，但尚未支持生命周期计划、阶段模式或归档编辑授权。
- 赛事表已有 `registration_start_at`、`registration_end_at`、`start_at`、`end_at`，这些字段当前主要用于展示，不会自动改变生命周期。
- 赛事管理页已有基本信息编辑、生命周期展示和手动推进区域，但没有阶段时间编辑控件。
- 服务端已有 worker 模块和 Tokio 运行环境，可新增生命周期后台 worker；当前没有赛事生命周期计划表或历史表。
- 报名、团队锁定、成绩提交、排行榜发布和谱面公开均依赖 `lifecycle`，生命周期变更必须继续复用排行榜重算及现有业务语义。

## Proposed Changes

### 1. 生命周期模型与数据库

文件范围：

- `crates/database/src/entities/tournament.rs`
- `crates/migrator/src/migrations/m_*.rs`
- `crates/migrator/src/migrations/mod.rs`
- `crates/migrator/src/lib.rs`

新增增量迁移，在 `tournament` 表增加：

- 四个阶段的调度模式字段：报名、比赛、评审、结束分别为 `manual` 或 `scheduled`，默认 `manual`。
- 四个阶段的开启时间字段：`registration_at`、`running_at`、`review_at`、`finished_at`，使用 `TIMESTAMPTZ`，仅对应 `scheduled` 模式时生效。
- `organizer_can_edit_archived` 布尔字段，默认 `false`，表示 DevOps 是否授权该赛事组织者修改归档赛事。

保留现有四个时间字段用于赛事业务时间展示和兼容；阶段调度时间作为生命周期控制字段，避免把报名结束或赛事结束时间与状态触发时间混为一谈。实体、序列化字段和前端模型同步增加对应枚举与可选时间字段。

生命周期规则调整为：

- 当前状态不是 `archived` 时，允许切换到任意生命周期状态，包括回退和跨阶段跳转。
- 当前状态为 `archived` 时，普通赛事组织者不能修改生命周期及调度配置。
- DevOps 可以修改归档赛事；赛事组织者只有在 `organizer_can_edit_archived = true` 时才可修改归档赛事的普通配置和生命周期。
- 手动将赛事设为 `archived` 后，不再允许赛事组织者自行改回其他状态；是否允许组织者编辑归档赛事由授权开关控制，但归档状态本身仍需要按权限保护，避免组织者绕过归档锁定。
- 自动推进只执行阶段顺序中的下一阶段，不跨阶段、不回退；配置时间早于当前时间的阶段在 worker 扫描时按顺序补齐推进，但遇到 `archived` 时停止。

为避免并发 worker 和手动操作互相覆盖，状态更新使用事务或带当前生命周期条件的更新，并在状态变化后复用现有排行榜重算逻辑。

### 2. 后端更新接口与权限

文件范围：

- `crates/server/src/routes/tournament/core.rs`
- `crates/server/src/routes/tournament/access.rs`
- `crates/server/src/routes/tournament/mod.rs`

扩展赛事更新 DTO，明确支持：

- 生命周期直接设置。
- 四个阶段的手动/定时模式。
- 四个阶段的开启时间。
- 赛事级归档编辑授权开关。

增加统一权限判断：

- 未归档赛事：赛事 Owner/Organizer 可编辑，保持现有 Judge 禁止编辑规则。
- 已归档赛事：DevOps 可编辑；Owner/Organizer 仅在赛事授权开关开启后可编辑普通字段。生命周期从归档改回其他状态只允许 DevOps，组织者不能解锁生命周期。
- 归档授权开关的修改只允许 DevOps，防止组织者自授予权限。

增加业务校验：

- 调度模式只能是 `manual` 或 `scheduled`。
- 定时模式必须提供对应开启时间；手动模式的时间可为空或保留但不触发自动推进。
- 开启时间不能早于创建时间；各阶段 scheduled 时间必须按生命周期顺序不晚于后续 scheduled 阶段，避免时间表逆序。
- 赛事结束阶段使用 `finished_at` 自动进入 `finished`；进入 `archived` 始终由人工操作，不新增自动归档。
- 保留现有团队人数、时间范围和排行榜重算校验。
- 对显式清空时间的请求使用可区分“未提供”和“传 null”的 DTO 表示，避免当前 `Option<T>` 无法清空字段的问题。

### 3. 生命周期后台 worker

文件范围：

- `crates/server/src/worker/mod.rs`
- 新增 `crates/server/src/worker/tournament_lifecycle.rs`
- `crates/server/src/lib.rs`

新增服务器启动时运行的后台任务：

- 以固定短间隔扫描启用定时阶段的未归档赛事。
- 按 `registration → running → review → finished` 顺序检查到期阶段。
- 仅在当前状态仍为前一阶段或更早状态时推进，使用数据库条件更新保证幂等。
- 每次状态实际变化后调用现有排行榜重算逻辑，并记录结构化日志；无变化时不重复重算。
- 任务失败只记录错误并继续下一轮，不影响 HTTP 服务。
- 服务关闭时通过 Tokio cancellation/shutdown 机制停止任务。

后台任务只负责自动阶段；归档仍必须由 DevOps 或有明确授权的赛事管理操作手动完成。

### 4. 赛事管理前端

文件范围：

- `web/src/routes/tournaments/[tournament]/admin/index.tsx`
- `web/src/lib/api/tournament.ts`
- `web/src/lib/models/tournament.ts`

重构生命周期管理区域：

- 用生命周期选择器替代仅显示“下一阶段”的单向推进按钮。
- 未归档时允许组织者选择任意生命周期并保存；归档状态显示锁定说明。
- DevOps 在归档赛事中显示可编辑控制；组织者根据赛事授权开关显示只读或可编辑状态。
- 增加 DevOps 专属的“允许赛事组织者修改归档赛事”开关。
- 增加四个阶段配置卡片，每张卡片包含阶段名称、手动/定时选择和 `datetime-local` 开启时间。
- 结束阶段支持定时进入“已结束”；归档阶段只显示手动操作，不提供自动归档配置。
- 修改模式时清晰显示时间是否生效；保存前在前端校验时间顺序并显示错误。
- 将浏览器本地时间按现有 Luxon 约定转换为 UTC 秒级 API 字段，读取时转换回本地时间。
- 保存成功后刷新赛事数据和生命周期显示，保留现有错误提示/loading 行为。

### 5. 生命周期展示与其他业务兼容

文件范围：

- `web/src/routes/admin/tournaments/index.tsx`
- `web/src/routes/tournaments/[tournament]/index.tsx`
- 受影响的服务端 tournament 查询、报名、成绩、排行榜和谱面公开逻辑

调整后台赛事列表和赛事详情：

- 显示当前生命周期、各阶段调度模式及下一次计划时间。
- 归档赛事显示锁定状态和组织者授权状态。
- 不改变现有报名、团队锁定、成绩提交、排行榜可见性和谱面公开的生命周期语义。
- 对生命周期任意跳转后复用现有状态消费逻辑，避免直接更新字段造成排行榜或权限状态不一致。

### 6. 多语言

文件范围：

- `web/src/lib/i18n/zh-cn.json`
- `web/src/lib/i18n/zh-tw.json`
- `web/src/lib/i18n/en-us.json`
- `web/src/lib/i18n/ja-jp.json`

新增生命周期管理、手动/定时、阶段开启时间、归档锁定、DevOps 授权和校验错误文案。

## Assumptions & Decisions

- 生命周期状态保持现有六个值，不新增状态。
- 未归档赛事支持任意状态跳转；自动推进只按顺序前进。
- 每个阶段独立选择手动或定时，阶段范围为报名、比赛、评审、结束。
- 结束阶段可自动进入 `finished`；`archived` 不自动进入，必须手动操作。
- 已归档后，DevOps 始终可以修改；赛事组织者默认不能修改，DevOps 可按赛事开启组织者修改授权。
- 归档授权开关只允许 DevOps 设置；赛事组织者不能通过编辑普通配置自行开启。
- 本次不新增生命周期历史审计表；以现有更新日志和结构化 worker 日志作为追踪手段。若后续需要完整审计，再单独增加历史表。
- 后台 worker 扫描间隔使用固定配置常量，不新增平台配置项；后续如需可配置化再扩展。
- 现有 `registration_start_at`、`registration_end_at`、`start_at`、`end_at` 继续表示业务时间范围，新的阶段开启字段专门负责生命周期调度。

## Verification

- 运行 Rust 格式检查、数据库/迁移/服务端编译检查和相关测试。
- 运行前端 lint、TypeScript diagnostics 和生产构建。
- 验证未归档赛事可从任意状态手动切换到任意状态。
- 验证归档后 Owner/Organizer、DevOps、授权开关三种权限组合。
- 验证四个阶段分别使用手动和定时模式时的前端保存、读取和 UTC 转换。
- 验证后台 worker 到期推进、重启后补偿推进、重复扫描幂等和归档停止推进。
- 验证自动进入 `finished` 后不会自动进入 `archived`。
- 验证报名、团队锁定、成绩提交、排行榜发布和谱面公开行为没有回归。
- 运行 `git diff --check`，确认无空白错误。
