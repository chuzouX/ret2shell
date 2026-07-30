# 赛段与谱面发布设计计划

## Summary

在赛事进入 `running` 生命周期后，引入独立的“当前赛段”控制和赛段内谱面发布策略。组织者可以手动选择当前赛段，系统根据赛段时间提供自动切换建议；当手动选择与时间线冲突时，前端弹窗展示冲突详情并由组织者确认。

每个赛段独立配置：

* 当前赛段内谱面的公开对象：所有访问者、已通过报名的参赛者、赛事 staff，可多选。

* 公开时机：立即公开、进入该赛段时公开、赛段结束时公开。

* 赛段结束方式：到达下一赛段时结束、设置结束时间自动结束、组织者手动结束。

* 手动提前发布和撤回。

staff 始终可以看到全部赛事谱面。已经结束的赛段按该赛段配置向所有访问者或 approved 参赛者开放。新赛段规则优先控制赛事内展示；现有 `private/public/after_archive` 继续控制公共谱面库发布，作为没有新赛段规则或公共库场景的兜底。

## Current State Analysis

当前代码已经具备以下基础：

* 赛事生命周期位于 `crates/database/src/entities/tournament.rs`，支持 `draft`、`registration`、`running`、`review`、`finished`、`archived`。

* 生命周期更新接口和后台推进 worker 位于 `crates/server/src/routes/tournament/core.rs` 和 `crates/server/src/worker/tournament_lifecycle.rs`。

* 赛段实体 `crates/database/src/entities/tournament_round.rs` 目前只有名称、说明、排序、开始时间和结束时间。

* 赛段管理接口位于 `crates/server/src/routes/tournament/core.rs`，目前仅执行赛事 staff 权限校验，没有赛段发布规则。

* 新赛事谱面关联实体 `crates/database/src/entities/tournament_chart_library.rs` 已有 `private`、`public`、`after_archive` visibility 字段。

* 赛事谱面读取位于 `crates/server/src/routes/chart_library.rs`，当前会直接返回赛事谱面关联和内部字段，没有按访问者身份裁剪。

* 赛事谱面管理位于 `web/src/routes/tournaments/[tournament]/admin/index.tsx`，已有赛段、谱面关联和谱面 visibility 的管理入口。

* 赛事谱面展示位于 `web/src/routes/tournaments/[tournament]/charts/index.tsx`，当前依赖完整赛事谱面关联响应。

* 报名状态 `approved` 位于 `crates/database/src/entities/registration.rs`，但目前没有公开的 approved 参赛者访问上下文；需要统一用于谱面过滤。

当前主要缺口：

* 没有赛事当前赛段字段。

* 没有赛段发布对象、发布时机、结束方式、已发布/已结束状态字段。

* 生命周期 worker 不处理赛段切换或赛段发布。

* 赛事谱面接口没有区分 staff、approved 参赛者和普通访问者。

* 前端没有赛段时间线、当前赛段选择、冲突确认、手动发布和撤回流程。

## Proposed Changes

### 1. 数据模型与迁移

修改 `crates/database/src/entities/tournament.rs`：

* 增加 `current_round_id: Option<i64>`，表示赛事当前运行赛段。

* 增加 `round_control_mode`，固定为手动优先并保留时间辅助语义；当前版本不提供完全自动接管模式。

* 相关字段使用 SeaORM 现有 ActiveEnum 风格，并保留 nullable 语义。

修改 `crates/database/src/entities/tournament_round.rs`：

* 增加 `release_audience: Json`，存储可多选的 `public`、`participants`、`staff` audience 列表。

* 增加 `release_timing`：`immediate`、`on_enter`、`on_end`。

* 增加 `end_mode`：`on_next_round`、`at_time`、`manual`。

* 增加 `release_at: Option<DateTime<Utc>>`，仅 `release_timing=on_enter` 或 `end_mode=at_time` 时按场景使用；前端和后端分别校验对应条件。

* 增加 `started_at: Option<DateTime<Utc>>`、`ended_at: Option<DateTime<Utc>>`，记录实际进入和结束时间。

* 增加 `released_at: Option<DateTime<Utc>>`，记录首次发布时间。

* 增加 `manually_released: bool`，表示组织者是否执行了提前发布。

* 增加 `manually_ended: bool`，表示组织者是否手动结束赛段。

* 增加 `release_version: i64` 或等价更新时间字段，用于发布/撤回后的缓存失效和并发更新判断。

修改 `crates/database/src/entities/tournament_chart_library.rs`：

* 保留现有 `visibility`，不立即删除或重命名，避免破坏公共谱面库发布语义。

* 通过 `round_id` 关联赛段，并由统一访问决策函数解释赛事内可见性。

* 新赛段规则存在时，`visibility` 不再直接决定赛事内普通访问者能否查看；它只继续决定公共谱面库是否发布。

新增迁移 `crates/migrator/src/migrations/m_YYYYMMDD_000008_add_round_release_control.rs`，并在 `crates/migrator/src/migrations/mod.rs` 和 `crates/migrator/src/lib.rs` 注册。

迁移默认值：

* `tournament.current_round_id = NULL`。

* 既有赛段 `release_audience = ["staff"]`，避免旧赛事在未配置规则时意外公开谱面。

* 既有赛段 `release_timing = on_enter`。

* 既有赛段 `end_mode = on_next_round`。

* `release_at`、`started_at`、`ended_at`、`released_at = NULL`。

* `manually_released = false`、`manually_ended = false`。

### 2. 统一赛段发布决策

新增 `crates/server/src/routes/tournament/round_visibility.rs` 或放入现有赛事模块，集中实现以下逻辑：

* 定义访问上下文：`Staff`、`ApprovedParticipant`、`Public`。

* staff 始终允许查看赛事所有赛段谱面，包括未开始、未发布和已撤回的谱面。

* 普通访问者只有在赛段规则包含 `public` 且发布条件满足时才能查看。

* approved 参赛者在赛段规则包含 `participants` 且发布条件满足时才能查看。

* `staff` audience 只用于明确记录 staff 范围，不改变 staff 永远可见的规则。

* 发布条件计算：

  * `immediate`：赛段配置保存后立即满足发布条件。

  * `on_enter`：赛事当前赛段等于该赛段，或该赛段已经进入过并记录 `started_at`。

  * `on_end`：赛段 `ended_at` 已存在。

* 赛段结束条件计算：

  * `on_next_round`：手动选择下一赛段，或系统确认进入下一赛段时结束当前赛段。

  * `at_time`：当前时间达到 `release_at`，由后台任务标记结束。

  * `manual`：组织者执行结束动作后标记结束。

* 手动提前发布设置 `manually_released=true`，发布决策立即满足，但仍遵守 audience。

* 手动撤回清除 `manually_released` 和 `released_at`，但如果赛段已经进入或结束，不允许撤回自动发布状态；未开始且只是提前发布的赛段允许撤回。

* 所有查询都只返回已经通过访问决策的谱面，不向普通用户返回 `private` 谱面、权重、内部排序和 metadata 等 staff 字段。

### 3. 后端 API 与权限

在 `crates/server/src/routes/tournament/core.rs` 增加赛事当前赛段和赛段动作接口，路由继续注册在 `crates/server/src/routes/tournament/mod.rs`：

* `PATCH /tournaments/{id}/rounds/{round_id}`：更新赛段名称、时间、发布 audience、发布时机和结束方式。

* `POST /tournaments/{id}/rounds/{round_id}/enter`：组织者手动进入赛段；若与时间建议冲突，返回结构化冲突信息，前端展示确认后再次提交 `force=true`。

* `POST /tournaments/{id}/rounds/{round_id}/release`：手动提前发布赛段谱面。

* `POST /tournaments/{id}/rounds/{round_id}/withdraw-release`：撤回尚未进入且仅由手动发布产生的公开状态。

* `POST /tournaments/{id}/rounds/{round_id}/end`：手动结束赛段。

* `GET /tournaments/{id}/rounds`：对 staff 返回完整配置；普通访问者只返回赛段公开需要的安全字段；报名者根据权限返回可见赛段信息。

* `GET /tournaments/{id}/charts` 与 `GET /tournaments/{id}/chart-library`：统一经过 round visibility 决策，保持 staff 管理端完整数据和普通用户安全投影。

权限要求：

* Owner、Organizer、DevOps 可修改未归档赛事赛段配置和执行发布动作。

* 归档赛事遵循现有赛事编辑授权；只有允许编辑归档赛事的组织者或 DevOps 可修改普通赛段字段。

* 只有 DevOps 能重新打开归档赛事或修改归档后生命周期。

* Judge 只读已发布内容，不允许改变赛段配置。

* 后端所有动作使用当前赛事状态和赛段状态条件更新，避免 worker 与手动操作互相覆盖。

校验规则：

* 赛段必须属于当前赛事。

* `release_audience` 至少包含一个值，值只能是 `public`、`participants`、`staff`。

* `release_timing=on_enter` 时不要求固定时间，由当前赛段进入动作触发；`immediate` 不允许依赖未来时间。

* `end_mode=at_time` 必须提供 `release_at`，且不得早于赛事创建时间和赛段开始时间。

* 赛段时间必须按 `order_index` 顺序递增，赛段结束时间不能早于开始时间。

* 当前赛段只能在赛事为 `running` 时设置；进入 `review`、`finished` 或 `archived` 时，当前赛段要么自动结束，要么由后端拒绝未完成的切换并返回待处理信息。

* 不允许手动进入已经结束的赛段，除非 DevOps 执行明确的赛事修复操作；普通组织者只能进入未结束的后续赛段。

### 4. 后台自动辅助任务

扩展 `crates/server/src/worker/tournament_lifecycle.rs`，或新增 `crates/server/src/worker/tournament_round.rs`：

* 每轮扫描 `running` 赛事及其赛段。

* 对 `end_mode=at_time` 且到期的赛段执行结束标记，并依据 `release_timing=on_end` 发布谱面。

* 对时间已到但当前赛段仍由手动选择控制的情况，不自动覆盖 `current_round_id`，只写入待处理建议状态或通过赛事详情接口计算冲突。

* 对生命周期从 `running` 进入 `review/finished` 的赛事，自动结束当前赛段，除非该赛事配置明确要求手动结束并记录异常状态供管理端处理。

* 每次更新使用 `WHERE tournament_id=? AND id=? AND ended_at IS NULL` 或等价条件，保证幂等。

* 不自动执行归档，也不替代组织者的手动赛段选择。

### 5. 管理端交互

修改 `web/src/routes/tournaments/[tournament]/admin/index.tsx`：

* 在现有赛事生命周期控制区增加“当前赛段”卡片和赛段时间线。

* running 状态下显示所有赛段的状态：未开始、当前、已结束、待处理冲突。

* 提供“进入赛段”按钮；点击后展示：当前赛段、时间建议赛段、开始时间、可能提前公开的谱面范围，确认后执行。

* 提供“立即公布赛段谱面”按钮和“撤回提前公布”按钮，撤回按钮仅在后端允许时显示。

* 提供“结束赛段”按钮；结束动作显示结束原因选择：手动结束、按时间结束、进入下一赛段。

* 赛段配置表单支持：

  * 公开对象多选：所有访问者、approved 参赛者、staff。

  * 公开时机：立即、进入赛段、赛段结束。

  * 结束方式：进入下一赛段、指定时间、手动。

  * 指定时间输入。

* 将赛段配置保存与赛事基本信息、赛事生命周期保存分开，避免一个保存动作同时修改多个高风险状态。

* 对时间冲突使用独立确认弹窗，不使用浏览器原生 `confirm`。

* 普通访问者不在管理接口中看到 `private` 谱面详情、谱面权重和内部 metadata。

修改 `web/src/lib/api/tournament.ts`：

* 增加 round 配置、进入、发布、撤回、结束 API 方法。

* 为冲突响应定义结构化类型，包括 `current_round`、`suggested_round`、`reason`、`affected_charts`。

* 为普通赛事谱面响应定义安全投影类型，避免复用管理端完整 `TournamentChartLibrary` 类型。

修改 `web/src/lib/models/tournament.ts`：

* 增加 `RoundReleaseAudience`、`RoundReleaseTiming`、`RoundEndMode` 类型。

* 扩展 `TournamentRound`，加入发布配置、当前/结束状态和时间字段。

* 增加 `TournamentRoundConflict`、`TournamentChartVisibilityDecision` 类型。

### 6. 参赛者与公众展示

修改 `crates/server/src/routes/chart_library.rs`：

* 赛事谱面查询统一调用 round visibility 决策。

* 赛事 staff 查询保留完整管理字段。

* approved 参赛者和普通访问者只收到当前身份允许的谱面安全字段。

* 继续保留公共谱面库查询中 `public/after_archive/private` 的公共库语义；赛段发布只控制赛事内部展示。

修改 `web/src/routes/tournaments/[tournament]/charts/index.tsx`：

* 未发布赛段显示“谱面尚未公布”，不显示谱面 metadata。

* 对 approved 参赛者和普通访问者显示其身份允许的赛段和谱面。

* 已结束赛段根据 audience 配置显示给所有访问者或 approved 参赛者。

* staff 管理入口保留完整谱面列表和发布状态。

### 7. 国际化

同步更新：

* `web/src/lib/i18n/zh-cn.json`

* `web/src/lib/i18n/zh-tw.json`

* `web/src/lib/i18n/en-us.json`

* `web/src/lib/i18n/ja-jp.json`

新增文案覆盖：

* 当前赛段、进入赛段、结束赛段。

* 公开对象和公开时机。

* 立即公开、进入时公开、结束时公开。

* 所有访问者、参赛者、staff。

* 待处理时间冲突、时间建议、受影响谱面数量。

* 立即公布、撤回公布、撤回限制。

* 谱面尚未公布、赛段已结束、无权限查看。

### 8. 测试与兼容

后端测试：

* 赛段属于赛事校验。

* audience 多选序列化和非法值拒绝。

* immediate、on\_enter、on\_end 三种发布时机。

* on\_next\_round、at\_time、manual 三种结束方式。

* staff、approved participant、public 三类访问结果。

* 手动发布、撤回、进入和结束的状态迁移。

* worker 结束赛段的幂等和并发条件更新。

* running 进入 review/finished 时当前赛段处理。

* 旧赛事迁移后默认不意外公开谱面。

前端验证：

* 赛事管理页可以选择当前赛段并确认冲突。

* 发布、撤回、结束按钮的权限和状态正确。

* 普通访问者、approved 参赛者、staff 看到不同谱面结果。

* 四套翻译 key 同步，Biome lint 和 production build 通过。

执行命令：

```text
cargo +nightly fmt --all -- --check
cargo check -p r2s-database -p r2s-migrator -p r2s-server
cargo test -p r2s-database -p r2s-server
corepack pnpm@10.11.0 -C web lint
corepack pnpm@10.11.0 -C web build
git diff --check
```

## Assumptions & Decisions

* 本次设计范围是赛段和赛段内谱面发布，不新增赛事全局公开状态，也不改造公开选手列表。

* staff 永远可见全部赛事谱面；`staff` audience 作为配置记录保留，但不会限制 staff。

* audience 使用多选存储；`public` 的访问范围自然包含 approved 参赛者，但后端仍保留显式 audience 值以便展示配置和未来扩展。

* 当前赛段采用手动优先；时间只生成建议和冲突提示，不会无确认覆盖组织者选择。

* 赛段结束默认由进入下一赛段触发，也可以选择指定时间或手动结束。

* 手动提前发布允许撤回，但只有在赛段尚未进入、且公开状态仅来自手动发布时允许撤回；已进入或已结束的赛段不可撤回。

* 新赛段规则优先用于赛事内部谱面展示；旧 `visibility` 字段继续用于公共谱面库发布，避免影响已有公共库数据。

* 不新增第三方依赖，复用现有 Axum、SeaORM、Tokio、SolidJS、Ark UI 和翻译机制。

* 需要先统一旧 `chart` 表和新 `tournament_chart_library` 的赛事谱面读取语义，再实施按身份过滤；否则旧谱面接口会绕过新发布规则。

## Verification

完成实现后按以下顺序验证：

1. 检查迁移能在已有数据库上执行，默认数据不会让旧赛事谱面意外公开。
2. 运行 Rust 格式检查、目标 crate 编译和生命周期/赛段相关测试。
3. 验证 worker 与手动赛段操作的并发条件更新不会覆盖彼此。
4. 使用 staff、approved participant、普通访问者三种身份请求赛事赛段和谱面接口，确认字段投影和可见性正确。
5. 在管理页验证当前赛段选择、时间冲突确认、提前发布、撤回和结束流程。
6. 运行四套翻译检查、前端 diagnostics、Biome lint、production build 和 `git diff --check`。

