# 赛事内容可见性设计

## 目标

赛事可见性需要同时解决四个问题：

- 赛事是否出现在公开列表中。
- 赛事详情、赛程结构和报名信息是否可以查看。
- 赛事谱面何时对工作人员、参赛者和公众开放。
- 成绩和排行榜在审核、冻结和结束后的展示方式。

可见性判断必须由后端统一执行，前端只负责根据返回状态改善导航和空状态。隐藏内容不能仅依靠前端不显示按钮来保护。

## 核心原则

### 发现与访问分离

赛事的“可发现性”只决定是否出现在赛事列表、搜索和推荐中，不决定赛事内部资源是否公开。

赛事内容访问还要同时经过：

1. 赛事生命周期判断。
2. 资源发布状态判断。
3. 当前用户角色判断。
4. 报名状态判断。

### 默认拒绝

新建赛事、赛段、谱面关联和成绩数据默认不公开。只有明确满足发布条件的内容才进入公开响应。

### 不泄露隐藏资源

无权访问的赛事、赛段和谱面统一返回 `404`，避免通过 `403`、标题、数量或错误信息泄露资源存在性。

对已经有权访问赛事但内容尚未发布的用户，可以返回明确的“尚未发布”状态；响应中不能包含标题、谱师、定数、封面、权重或原始元数据。

## 角色

| 角色 | 说明 |
| --- | --- |
| `public` | 未登录用户 |
| `user` | 已登录但没有有效报名 |
| `participant` | 当前赛事存在 `approved` 报名 |
| `judge` | 赛事裁判 |
| `organizer` | 赛事组织者 |
| `owner` | 赛事所有者 |
| `host` | 平台级赛事管理员 |

赛事 staff 权限和平台 Host 权限分开判断。Judge 不应因为能够查看成绩而获得赛事配置、谱面库维护或生命周期推进权限。

## 赛事可见性

建议为 `tournament` 增加 `visibility` 字段：

```text
private   仅 owner、organizer、judge 可访问
unlisted  不出现在公开列表，但知道链接的公众可以访问已发布内容
public    出现在公开列表，内容仍受生命周期和发布规则限制
```

### 赛事与生命周期组合规则

| 生命周期 | `private` | `unlisted` | `public` |
| --- | --- | --- | --- |
| `draft` | staff | staff | staff，公开列表显示“筹备中”或不展示，二选一 |
| `registration` | staff | 公开赛事信息和报名状态 | 公开赛事信息和报名状态 |
| `running` | staff | 公开已发布内容 | 公开已发布内容 |
| `review` | staff | 公开已发布结果和排行榜 | 公开已发布结果和排行榜 |
| `finished` | staff | 公开已发布结果和排行榜 | 公开已发布结果和排行榜 |
| `archived` | owner、organizer、judge、host | 按赛事配置保留访问 | 按赛事配置保留访问 |

推荐默认行为：`draft` 强制私有，即使字段为 `public` 也不能让普通用户访问草稿内容。正式发布必须由 owner 或 organizer 显式推进到 `registration`。

### 赛事详情的公开字段

公众或普通用户读取赛事详情时，只返回：

- 名称、简介、描述、封面。
- 生命周期和公开时间。
- 比赛模式、报名时间、开始和结束时间。
- 是否允许报名、是否已经结束。

以下字段只返回给 staff：

- owner ID、staff 详情。
- 内部配置和编辑状态。
- 未发布赛段数量、备用谱面数量。
- 赛事内部元数据。

## 赛段可见性

赛段需要独立于赛事增加发布配置。建议字段：

```text
visibility: hidden | participants | public
release_mode: manual | round_start
release_at: optional timestamp
```

含义：

- `hidden`：仅 staff 可见。
- `participants`：已批准参赛者和 staff 可见。
- `public`：满足发布时间后公众可见。
- `manual`：必须由 organizer 显式发布。
- `round_start`：到赛段 `start_at` 后自动发布。
- `release_at`：到指定时间后发布；时间未到时保持隐藏。

赛段发布条件为：

```text
visible = visibility != hidden
  && (release_mode == manual && released_at != null
      || release_mode == round_start && now >= start_at
      || release_at != null && now >= release_at)
```

若赛段没有 `start_at`，`round_start` 不应自动发布，后端返回配置错误给 staff，并对普通用户保持隐藏。

## 谱面可见性

赛事谱面关联需要与公共谱面库分离。公共谱面库记录可以公开，但赛事关联记录增加：

```text
visibility: hidden | participants | public
release_mode: inherit_round | manual | scheduled
release_at: optional timestamp
```

### 推荐默认值

- 赛事草稿中的所有谱面：`hidden`。
- 赛事进入报名阶段：继承赛段发布状态，但不自动公开谱面详情。
- 赛事运行时：只对当前允许作答的参赛者公开当前赛段谱面。
- 赛事结束或 organizer 显式发布后：才对公众公开谱面详情。

### 谱面展示分层

谱面不能只用一个布尔值控制所有字段，建议按访问者返回不同投影：

| 状态 | 可返回内容 |
| --- | --- |
| 未发布 | 不返回谱面记录；对已进入赛事的用户显示“本阶段尚未开放” |
| 参赛者可见 | 标题、曲师、谱师、难度、定数、封面、谱面 ID |
| 公众可见 | 上述字段，以及允许公开的来源和元数据 |
| staff | 全部字段，包括内部权重、来源外部 ID、审核字段 |

权重、排序、备用标记、内部 metadata 和未发布谱面的数量永远不应返回给公众或普通参赛者。

### 赛事谱面池与当前赛段

赛事公开谱面列表只返回当前用户有权访问的谱面：

- 公众：只返回 `public` 且已到发布时间的谱面。
- 普通登录用户：与公众相同。
- approved participant：返回已发布的 `participants` 和 `public` 谱面。
- judge、organizer、owner、host：返回全部谱面，并包含发布状态。

如果参赛者需要在比赛开始前查看谱面，应把规则设置为 `participants`；如果谱面需要比赛开始才揭示，应使用 `round_start`，而不是在前端临时隐藏。

## 成绩可见性

成绩和谱面可见性独立判断。

### 成绩读取

| 访问者 | 可读取内容 |
| --- | --- |
| 公众 | 不读取原始成绩，只读取公开排行榜 |
| 普通登录用户 | 不读取原始成绩 |
| approved participant | 自己的有效成绩和处理状态；不读取他人成绩 |
| judge、organizer、owner | 赛事全部成绩和审核记录 |
| host | 平台范围内按权限读取 |

普通用户读取自己的成绩时，`pending` 可以显示为“审核中”，`rejected` 显示为“未通过”，`voided` 显示为“已作废”；这些状态不能进入公开排行榜。

成绩查询必须校验报名状态，不能仅因为存在报名记录就允许读取。`withdrawn` 和 `rejected` 报名不应继续获得参赛成绩入口。

## 排行榜可见性

保留现有 `live`、`frozen`、`after_end`，但定义为公开快照策略：

```text
live      公开最近一次成功计算的 approved 成绩快照
frozen    公开最后一次手动发布的快照，不随后台重算变化
after_end 赛事进入 finished 或 archived 后才公开快照
```

建议增加快照状态：

```text
public_snapshot: boolean
published_at: optional timestamp
stale: boolean
```

规则：

- 计算失败时保留最后一次公开快照，并返回 `stale = true`。
- 没有任何公开快照时返回结构化状态 `not_published`，不要返回普通空数组。
- staff 可以通过单独的实时视图读取未公开快照。
- `frozen` 不应依赖“恰好存在某一条历史公开快照”，应明确保存冻结快照。

## 不可见时的用户体验

### 赛事不存在或无权访问

后端返回 `404`，前端显示：

```text
赛事不存在，或当前不可访问
```

不显示赛事名称、阶段数量或是否曾经存在。

### 赛事存在但内容尚未发布

对已获得赛事访问权的用户返回结构化状态：

```json
{
  "state": "not_published",
  "release_at": null,
  "message_key": "tournament.content.notPublished"
}
```

前端显示：

```text
该内容尚未发布
组织者将在赛程开放后公布
```

不渲染空白列表、不显示数量 0、不显示谱面占位卡片中的标题。

### 参赛者等待赛段开放

如果当前用户是 approved participant，但赛段尚未开始：

```text
本赛段尚未开放
开放时间：2026-07-28 20:00
```

只有在 `release_at` 明确允许公开时才显示具体时间；否则只显示“开放时间由赛事组织者决定”。

### 排行榜不可见

根据状态区分：

- `not_published`：排行榜尚未发布。
- `after_end`：赛事结束后公开排行榜。
- `no_data`：已发布但暂无有效成绩。
- `stale`：显示最近公开结果，并标记数据正在重算。

## 接口设计

### 统一访问上下文

后端每个赛事资源读取前构造统一上下文：

```text
TournamentAccess {
  tournament
  role: public | user | participant | judge | organizer | owner | host
  lifecycle
  registration_status
}
```

资源处理器只调用统一策略，不再自行判断 `token.id`、staff 或报名状态。

建议新增内部策略函数：

```text
can_view_tournament
can_view_round
can_view_chart
can_view_result
can_view_leaderboard
```

### API 响应

- 无权访问或资源不存在：`404`。
- 已登录但未报名：`403` 仅用于需要登录或需要有效报名的写操作。
- 内容尚未发布：读取接口返回 `200` 加结构化 `not_published`，或返回 `404`；同一个资源类型只能选一种方案。
- 推荐：列表接口返回空的发布状态对象，详情接口返回 `200` 的 `not_published` 状态，避免前端误判为网络错误。

建议统一响应类型：

```json
{
  "data": [],
  "visibility": {
    "state": "published | not_published | participants_only | staff_only",
    "release_at": null
  }
}
```

## 数据库改造建议

第一阶段新增字段，不删除旧字段：

```text
tournament.visibility
tournament.publish_at
tournament_round.visibility
tournament_round.release_mode
tournament_round.release_at
tournament_round.released_at
tournament_chart_library.visibility
tournament_chart_library.release_mode
tournament_chart_library.release_at
tournament_chart_library.released_at
```

字段默认值：

- 新赛事：`private`。
- 新赛段：`hidden`、`manual`。
- 新赛事谱面关联：`inherit_round`。

现有赛事兼容策略：

- 已经处于 `registration` 或更晚阶段的赛事迁移为 `public`。
- 已经存在的赛段和赛事谱面关联迁移为 `public`，避免上线后突然对现有用户隐藏。
- `draft` 赛事迁移为 `private`。

## 前端规则

前端导航按以下条件显示：

- 概览：赛事可见时显示。
- 谱面：存在至少一条当前用户可见谱面时显示；否则显示“谱面尚未发布”。
- 成绩：仅登录用户显示；非 approved participant 显示只读说明，不显示提交表单。
- 排行榜：始终可以进入，但根据返回状态显示“尚未发布”“结束后公开”或数据内容。
- 管理：仅 organizer、owner 显示完整入口；judge 使用单独的裁判视图。

按钮状态必须与后端一致：

- 报名按钮根据 lifecycle 和报名状态显示。
- 创建队伍、加入队伍和成绩提交按钮必须要求 `approved` 报名。
- 赛段和谱面发布按钮只对 organizer、owner 显示。
- staff 页面明确区分“查看全部”和“可修改”。

## 实施顺序

### 第一阶段：统一访问策略

1. 增加 `TournamentAccess` 和资源访问策略。
2. 先保护 `draft` 赛事及其赛段、标签、谱面、队伍读取接口。
3. 修正成绩读取的报名状态校验。
4. 限制全局谱面库写入到 Host 或谱面库维护角色。

### 第二阶段：赛事和赛段发布

1. 增加赛事 `visibility`。
2. 增加赛段发布字段和发布/撤回接口。
3. 前端增加赛事发布状态和不可见状态。

### 第三阶段：谱面阶段性发布

1. 增加赛事谱面关联的发布字段。
2. 实现继承赛段、手动发布和定时发布。
3. 谱面列表按访问者返回不同字段投影。
4. 增加“尚未发布”和“本阶段尚未开放”页面状态。

### 第四阶段：排行榜和成绩状态

1. 增加 `not_published`、`no_data`、`stale` 状态。
2. 明确冻结快照。
3. 分离 staff 实时榜和公众榜。
4. 修正前端导航和成绩提交状态。

### 第五阶段：管理权限收敛

1. organizer/owner 管理赛事和谱面发布。
2. judge 管理成绩和审核，不管理赛事结构。
3. Host 管理全局谱面库和平台级赛事。
4. 增加 API 和前端权限测试。

## 验收标准

- 公众无法通过列表、详情或直接 URL 读取草稿赛事。
- 赛事公开后，未发布赛段和谱面不会泄露标题、数量、排序或元数据。
- approved participant 只能看到已对参赛者发布的谱面。
- 普通用户不能读取他人成绩，也不能读取已撤回报名的成绩。
- pending、rejected、voided 成绩不会进入公开排行榜。
- `live`、`frozen`、`after_end` 的公开结果有明确状态，不再用空列表表达所有情况。
- 赛事谱面解除关联不会删除公共谱面库记录。
- Host、owner、organizer、judge 的读写边界有后端测试覆盖。
