# Phira API 文档

## 目录

- [通用说明](#通用说明)
- [鉴权与账户](#鉴权与账户)
- [用户](#用户)
- [谱面](#谱面)
- [成绩](#成绩)
- [收藏集](#收藏集)
- [活动](#活动)
- [消息](#消息)
- [OAuth](#oauth)
- [仪表盘](#仪表盘)
- [杂项](#杂项)

---

## 通用说明

### Base URL

```
https://phira.5wyxi.com
```

### 鉴权方式

大多数端点需要 Bearer Token 鉴权：

```
Authorization: Bearer <access_token>
```

Token 通过 `POST /login` 获取，有效期为 6 小时。Refresh Token 有效期为 30 天。

### 通用响应格式

成功时返回 JSON 数据；失败时返回：

```json
{
  "error": "错误描述"
}
```

### 分页参数

列表端点通常支持以下查询参数：

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `page` | number | 1 | 页码，从 1 开始 |
| `pageNum` | number | 30 | 每页数量，最大 30 |
| `order` | string | - | 排序字段，如 `id asc`、`rating desc`、`-created` |
| `search` | string | - | 搜索关键词 |
| `tags` | string | - | 标签过滤，逗号分隔 |

分页响应格式：

```json
{
  "count": 100,
  "results": [...]
}
```

### 通用数据模型

#### UserView

```json
{
  "id": 123,
  "name": "username",
  "avatar": "https://...",
  "badges": ["badge1"],
  "language": "zh-CN",
  "bio": "个人简介",
  "exp": 1000,
  "rks": 15.5,
  "joined": "2025-01-01T00:00:00Z",
  "lastLogin": "2025-01-01T00:00:00Z",
  "roles": 0,
  "banned": false,
  "loginBanned": false,
  "followerCount": 10,
  "followingCount": 5
}
```

#### ChartView

```json
{
  "id": 123,
  "name": "谱面名称",
  "level": "IN Lv.15",
  "difficulty": 15.5,
  "charter": "谱师",
  "composer": "曲师",
  "illustrator": "画师",
  "description": "简介",
  "ranked": false,
  "reviewed": true,
  "stable": false,
  "stableRequest": false,
  "illustration": "https://...",
  "preview": "https://...",
  "file": "https://...",
  "uploader": 123,
  "tags": ["tag1"],
  "rating": 4.5,
  "ratingCount": 100,
  "created": "2025-01-01T00:00:00Z",
  "updated": "2025-01-01T00:00:00Z",
  "chartUpdated": "2025-01-01T00:00:00Z"
}
```

#### RecordView

```json
{
  "id": 123,
  "player": 123,
  "chart": 123,
  "score": 1000000,
  "accuracy": 99.5,
  "perfect": 100,
  "good": 5,
  "bad": 0,
  "miss": 0,
  "speed": 1.0,
  "maxCombo": 105,
  "best": true,
  "bestStd": false,
  "mods": 0,
  "fullCombo": true,
  "time": "2025-01-01T00:00:00Z",
  "std": 15.5,
  "stdScore": 99.5
}
```

---

## 鉴权与账户

### POST /login

用户登录，获取访问令牌。

**请求体（二选一）：**

方式一：密码登录
```json
{
  "email": "user@example.com",
  "password": "password"
}
```

方式二：Refresh Token 刷新
```json
{
  "refreshToken": "<refresh_token>"
}
```

**响应：**

```json
{
  "id": 123,
  "token": "<access_token>",
  "refreshToken": "<refresh_token>",
  "expireAt": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
# 密码登录
curl -X POST https://phira.5wyxi.com/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# Refresh Token 刷新
curl -X POST https://phira.5wyxi.com/login \
  -H "Content-Type: application/json" \
  -d '{"refreshToken":"<refresh_token>"}'
```

---

### POST /register

用户注册。

**请求体：**

```json
{
  "email": "user@example.com",
  "name": "username",
  "password": "password"
}
```

**参数限制：**
- `name`: 4-30 个字符
- `password`: 6-32 个字符
- `email`: 合法邮箱格式

**响应：**

```json
{
  "expireAt": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/register \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","name":"username","password":"password"}'
```

---

### GET /me

获取当前登录用户信息。

**响应：** `DetailedUserView`（包含 `UserView` 所有字段 + `email`）

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/me \
  -H "Authorization: Bearer <access_token>"
```

---

### PATCH /me

修改当前用户信息。

**请求体（所有字段可选）：**

```json
{
  "name": "newname",
  "avatar": "<file_id>",
  "bio": "个人简介",
  "language": "zh-CN",
  "char": "shee"
}
```

**curl 示例：**

```bash
curl -X PATCH https://phira.5wyxi.com/me \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"newname","bio":"新简介"}'
```

---

### GET /me/subs

获取当前用户的邮件订阅设置。

**响应：**

```json
{
  "review": true,
  "stb": true
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/me/subs \
  -H "Authorization: Bearer <access_token>"
```

---

### PATCH /me/subs

修改当前用户的邮件订阅设置。

**请求体：**

```json
{
  "review": true,
  "stb": false
}
```

**curl 示例：**

```bash
curl -X PATCH https://phira.5wyxi.com/me/subs \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"review":true,"stb":false}'
```

---

### GET /me/char

获取当前用户角色信息。

**查询参数：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `locale` | string | `en-US` | 语言区域 |

**响应：** `Character` 对象

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/me/char?locale=zh-CN" \
  -H "Authorization: Bearer <access_token>"
```

---

### GET /me/chars

获取当前用户可用角色列表。

**查询参数：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `locale` | string | `en-US` | 语言区域 |

**响应：** `Character[]`

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/me/chars?locale=zh-CN" \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /me/identity-token

创建身份令牌（用于第三方服务验证用户身份）。

**响应：**

```json
{
  "token": "<access_token>",
  "refreshToken": "<refresh_token>",
  "expireAt": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/me/identity-token \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /edit/password

修改当前用户密码。

**请求体：**

```json
{
  "old": "旧密码",
  "new": "新密码"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/edit/password \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"old":"old_password","new":"new_password"}'
```

---

### POST /edit/avatar

修改当前用户头像。

**请求体：**

```json
{
  "file": "<file_id>"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/edit/avatar \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"file":"<file_id>"}'
```

---

### POST /delete-account

删除当前账户（提交删除申请）。

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/delete-account \
  -H "Authorization: Bearer <access_token>"
```

---

### GET /activate

邮箱激活链接（通过注册邮件访问）。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `token` | string | 激活令牌 |

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/activate?token=<token>"
```

---

### GET /unsubscribe

邮件退订链接（通过邮件访问）。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `token` | string | 退订令牌 |

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/unsubscribe?token=<token>"
```

---

### GET /reset-password

重置密码页面（Web 页面，返回 HTML）。

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/reset-password
```

---

### POST /reset-password/form

提交重置密码请求（发送重置邮件）。

**请求体（表单格式）：**

```
email=user@example.com
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/reset-password/form \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "email=user@example.com"
```

---

### POST /reset-password/do

执行密码重置。

**请求体：**

```json
{
  "token": "<reset_token>",
  "password": "新密码"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/reset-password/do \
  -H "Content-Type: application/json" \
  -d '{"token":"<reset_token>","password":"new_password"}'
```

---

### GET /reset-password/ui

重置密码 UI 页面（Web 页面，返回 HTML）。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `token` | string | 重置令牌 |

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/reset-password/ui?token=<token>"
```

---

### POST /logout-all

登出所有设备。

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/logout-all \
  -H "Authorization: Bearer <access_token>"
```

---

## 用户

### GET /user

获取用户列表，支持搜索和过滤。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `search` | string | 按用户名搜索 |
| `following` | number | 筛选关注某人的用户 ID |
| `followedBy` | number | 筛选被某人关注的用户 ID |
| `page` | number | 页码 |
| `pageNum` | number | 每页数量 |

**响应：** `PaginationR<UserView>`

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/user?search=test&page=1&pageNum=20"
```

---

### GET /user/{id}

获取指定用户信息。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 用户 ID |

**响应：** `GetUserR`（`UserView` + `following` 字段）

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/user/123
```

---

### POST /user/{id}/follow

关注或取消关注指定用户。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 用户 ID |

**请求体：**

```json
{
  "follow": true
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/user/123/follow \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"follow":true}'
```

---

### GET /user/{id}/stats

获取指定用户的统计信息。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 用户 ID |

**响应：**

```json
{
  "numRecords": 100,
  "avgAccuracy": 99.5
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/user/123/stats
```

---

### POST /user/{id}/report

举报指定用户。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 用户 ID |

**请求体：**

```json
{
  "reason": "举报原因（10-200字符）"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/user/123/report \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"reason":"这是举报原因，需要10到200个字符"}'
```

---

## 谱面

### GET /chart

获取谱面列表。

**查询参数：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `search` | string | - | 搜索谱面名称/简介，以 `#` 开头可按 ID 搜索 |
| `uploader` | number | - | 按上传者 ID 过滤 |
| `type` | number | 3 | 谱面类型：0=Ranked, 1=Unranked, 2=Unstable, 3=全部 |
| `division` | string | `regular` | 分区标签 |
| `rating` | string | - | 难度范围，如 `15.0,16.0` |
| `order` | string | `updated desc` | 排序：`id`、`created`、`updated`、`difficulty`、`name`、`rating` |
| `tags` | string | - | 标签过滤 |
| `page` | number | 1 | 页码 |
| `pageNum` | number | 30 | 每页数量 |

**响应：** `PaginationR<ChartView>`

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/chart?type=0&order=-rating&page=1&pageNum=20"
```

---

### GET /chart/multi-get

批量获取谱面信息。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `ids` | string | 谱面 ID 列表，逗号分隔，最多 100 个 |

**响应：** `ChartView[]`

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/chart/multi-get?ids=1,2,3"
```

---

### GET /chart/popular

获取热门谱面列表。

**查询参数：** 同分页参数

**响应：** `PaginationR<ChartView>`

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/chart/popular?page=1&pageNum=20"
```

---

### GET /chart/stable-requests

获取稳定化申请列表。

**查询参数：** 同分页参数

**响应：** `PaginationR<StableRequestR>`

```json
{
  "count": 10,
  "results": [
    {
      "chart": { ...ChartView },
      "approvedBy": ["user1", "user2"],
      "deniedBy": ["user3"],
      "status": "pending"
    }
  ]
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/chart/stable-requests?page=1"
```

---

### GET /chart/{id}

获取指定谱面详情。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**响应：** `ChartView`

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/chart/123
```

---

### PATCH /chart/{id}

编辑指定谱面信息（仅限上传者或管理员）。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**请求体：**

```json
{
  "file": "<file_id>",
  "created": "2025-01-01T00:00:00Z"
}
```

**响应：**

```json
{
  "updated": "2025-01-01T00:00:00Z",
  "chartUpdated": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
curl -X PATCH https://phira.5wyxi.com/chart/123 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"file":"<file_id>","created":"2025-01-01T00:00:00Z"}'
```

---

### DELETE /chart/{id}

删除指定谱面。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**curl 示例：**

```bash
curl -X DELETE https://phira.5wyxi.com/chart/123 \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /chart/{id}/rate

为指定谱面评分。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**请求体：**

```json
{
  "score": 8
}
```

**参数限制：** `score` 必须为 1-10 的整数

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/chart/123/rate \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"score":8}'
```

---

### GET /chart/{id}/rate

获取当前用户对指定谱面的评分。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**响应：**

```json
{
  "id": 123,
  "score": 8
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/chart/123/rate \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /chart/{id}/edit-tags

编辑指定谱面的标签。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**请求体：**

```json
{
  "tags": ["tag1", "tag2"]
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/chart/123/edit-tags \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"tags":["tag1","tag2"]}'
```

---

### POST /chart/{id}/req-stabilize

为指定谱面申请稳定化。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**响应：**

```json
{
  "updated": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/chart/123/req-stabilize \
  -H "Authorization: Bearer <access_token>"
```

---

### GET /chart/{id}/stabilize-status

获取指定谱面的稳定化状态。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**响应：**

```json
{
  "stable": false,
  "stableRequest": true,
  "approves": [{"id": 1, "name": "user1"}],
  "denies": [{"id": 2, "name": "user2"}],
  "history": [
    {
      "reviewer": 1,
      "reviewerName": "user1",
      "reviewerAvatar": "...",
      "chart": 123,
      "approve": true,
      "comment": null,
      "time": "2025-01-01T00:00:00Z"
    }
  ]
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/chart/123/stabilize-status
```

---

### GET /chart/{id}/verify-cksum

校验谱面文件完整性。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `checksum` | string | 文件 SHA256 校验和（hex 编码） |

**响应：**

```json
{
  "ok": true
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/chart/123/verify-cksum?checksum=abc123..."
```

---

### POST /chart/upload

上传谱面文件。

**请求体：**

```json
{
  "file": "<file_id>",
  "patch": {
    "name": "谱面名称",
    "level": "IN Lv.15",
    "charter": "谱师",
    "composer": "曲师",
    "illustrator": "画师",
    "intro": "简介"
  }
}
```

**响应：**

```json
{
  "id": 123,
  "created": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/chart/upload \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"file":"<file_id>","patch":{"name":"谱面名称"}}'
```

---

### GET /chart/collab/{id}

获取协作邀请信息。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 邀请哈希值 |

**响应：**

```json
{
  "chart": { ...ChartView },
  "role": "角色",
  "confirmed": false
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/chart/collab/<hash> \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /chart/collab/{id}/confirm

确认协作邀请。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 邀请哈希值 |

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/chart/collab/<hash>/confirm \
  -H "Authorization: Bearer <access_token>"
```

---

## 成绩

### GET /record

获取成绩列表。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `player` | number | 按玩家 ID 过滤 |
| `chart` | number | 按谱面 ID 过滤 |
| `order` | string | 排序：`time`、`score` |
| `page` | number | 页码 |
| `pageNum` | number | 每页数量 |

**响应：** `RecordView[]`（最多 20 条）

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/record?player=123&order=-time"
```

---

### GET /record/multi-get

批量获取成绩。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `ids` | string | 成绩 ID 列表，逗号分隔，最多 100 个 |

**响应：** `RecordView[]`

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/record/multi-get?ids=1,2,3"
```

---

### GET /record/query/{id}

获取指定谱面的排行榜。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**查询参数：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `includePlayer` | boolean | false | 是否包含玩家信息 |
| `std` | boolean | false | 是否使用标准分数排序 |
| `page` | number | 1 | 页码 |
| `pageNum` | number | 30 | 每页数量 |

**响应：**

```json
{
  "count": 100,
  "results": [
    {
      "id": 123,
      "player": 123,
      "chart": 123,
      "score": 1000000,
      "accuracy": 99.5,
      "playerName": "username",
      "playerAvatar": "...",
      "playerBadges": ["badge1"]
    }
  ]
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/record/query/123?includePlayer=true&std=true&page=1"
```

---

### GET /record/list15/{id}

获取指定谱面的前 15 名成绩。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**查询参数：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `std` | boolean | false | 是否使用标准分数 |

**响应：** `List15R[]`

```json
[
  {
    "id": 123,
    "player": 123,
    "chart": 123,
    "score": 1000000,
    "rank": 1
  }
]
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/record/list15/123?std=true"
```

---

### GET /record/best/{id}

获取当前用户在指定谱面的最佳成绩。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 谱面 ID |

**响应：**

```json
{
  "score": 1000000,
  "accuracy": 99.5,
  "fullCombo": true
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/record/best/123 \
  -H "Authorization: Bearer <access_token>"
```

---

### GET /record/get-pool/{id}

获取指定用户的成绩池。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 用户 ID |

**响应：**

```json
{
  "bestPool": [
    {"record": 123, "chart": 123, "rks": 15.5}
  ],
  "recentPool": [
    {"record": 456, "chart": 456, "rks": 14.5}
  ],
  "rks": 15.0
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/record/get-pool/123
```

---

### GET /record/{id}

获取指定成绩详情。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 成绩 ID |

**响应：** `RecordView`

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/record/123
```

---

### POST /play/upload

上传游戏成绩。

**请求体：**

```json
{
  "chart": 123,
  "token": "<record_token>",
  "chartUpdated": "2025-01-01T00:00:00Z"
}
```

**响应：**

```json
{
  "id": 123,
  "expDelta": 0.0,
  "newBest": false,
  "improvement": 0,
  "newRks": 15.5
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/play/upload \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"chart":123,"token":"<record_token>"}'
```

---

## 收藏集

### GET /collection

获取收藏集列表。

**查询参数：** 同分页参数

**响应：** `PaginationR<Collection>`

```json
{
  "count": 10,
  "results": [
    {
      "id": 123,
      "owner": 123,
      "name": "收藏集名称",
      "description": "描述",
      "created": "2025-01-01T00:00:00Z",
      "updated": "2025-01-01T00:00:00Z",
      "cover": "https://...",
      "public": true,
      "likes": 10
    }
  ]
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/collection?page=1&pageNum=20"
```

---

### POST /collection

创建收藏集。

**请求体：**

```json
{
  "name": "收藏集名称",
  "description": "描述",
  "charts": [1, 2, 3],
  "public": true
}
```

**参数限制：**
- `name`: 最多 20 字符
- `description`: 最多 1000 字符
- `charts`: 最多 100 个

**响应：** `DetailedCollection`

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/collection \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"收藏集","description":"描述","charts":[1,2,3],"public":true}'
```

---

### GET /collection/{id}

获取指定收藏集详情。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 收藏集 ID |

**响应：** `DetailedCollection`

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/collection/123
```

---

### PUT /collection/{id}

全量更新指定收藏集。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 收藏集 ID |

**请求体：**

```json
{
  "name": "收藏集名称",
  "description": "描述",
  "charts": [1, 2, 3],
  "public": true,
  "updated": "2025-01-01T00:00:00Z"
}
```

**响应：** `DetailedCollection`

**curl 示例：**

```bash
curl -X PUT https://phira.5wyxi.com/collection/123 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"新名称","description":"新描述","charts":[1,2],"public":true}'
```

---

### PATCH /collection/{id}

部分更新指定收藏集。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 收藏集 ID |

**请求体（三选一）：**

设置谱面列表：
```json
{
  "set": [1, 2, 3]
}
```

设置公开状态：
```json
{
  "public": true
}
```

设置封面：
```json
{
  "cover": 123
}
```

**响应：** `DetailedCollection`

**curl 示例：**

```bash
curl -X PATCH https://phira.5wyxi.com/collection/123 \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"public":true}'
```

---

### DELETE /collection/{id}

删除指定收藏集。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 收藏集 ID |

**curl 示例：**

```bash
curl -X DELETE https://phira.5wyxi.com/collection/123 \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /collection/{id}/report

举报指定收藏集。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 收藏集 ID |

**请求体：**

```json
{
  "reason": "举报原因（10-200字符）"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/collection/123/report \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"reason":"这是举报原因，需要10到200个字符"}'
```

---

### GET /collection/{id}/like

获取当前用户对指定收藏集的点赞状态。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 收藏集 ID |

**响应：**

```json
{
  "like": true
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/collection/123/like \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /collection/{id}/like

点赞或取消点赞指定收藏集。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 收藏集 ID |

**请求体：**

```json
{
  "like": true
}
```

**响应：**

```json
{
  "likes": 11
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/collection/123/like \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"like":true}'
```

---

## 活动

### GET /event

获取活动列表。

**查询参数：** 同分页参数

**响应：** `PaginationR<EventView>`

```json
{
  "count": 10,
  "results": [
    {
      "id": 123,
      "sid": "event-sid",
      "creator": 123,
      "name": "活动名称",
      "illustration": "https://...",
      "timeStart": "2025-01-01T00:00:00Z",
      "timeEnd": "2025-01-31T00:00:00Z",
      "locked": false
    }
  ]
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/event?page=1&pageNum=20"
```

---

### GET /event/{id}

获取指定活动详情。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 活动 ID |

**响应：** `EventView`

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/event/123
```

---

### GET /event/{id}/uml

获取指定活动的 UML 内容。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 活动 ID |

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `version` | string | 客户端版本号 |

**响应：** 纯文本 UML 内容

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/event/123/uml?version=1.0.0"
```

---

### POST /event/{id}/join

加入指定活动。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 活动 ID |

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/event/123/join \
  -H "Authorization: Bearer <access_token>"
```

---

### GET /event/{id}/status

获取指定活动的状态。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 活动 ID |

**响应：**

```json
{
  "joined": true,
  "rank": 1,
  "score": 1000
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/event/123/status \
  -H "Authorization: Bearer <access_token>"
```

---

### GET /event/{id}/list15

获取指定活动的排行榜前 15 名。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 活动 ID |

**响应：**

```json
[
  {
    "player": 123,
    "rank": 1,
    "score": 1000
  }
]
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/event/123/list15
```

---

## 消息

### GET /message/list

获取当前用户的消息列表。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `before` | string | 获取此时间之前的消息，ISO 8601 格式 |

**响应：** `MessageView[]`

```json
[
  {
    "id": 123,
    "title": "消息标题",
    "content": "消息内容",
    "author": "发送者",
    "time": "2025-01-01T00:00:00Z",
    "actions": [{"name": "查看", "action": "chart:123"}]
  }
]
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/message/list?before=2025-01-01T00:00:00Z" \
  -H "Authorization: Bearer <access_token>"
```

---

### GET /message/has_new

检查当前用户是否有新消息。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `checked` | string | 上次检查时间，ISO 8601 格式 |

**响应：**

```json
{
  "has": true
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/message/has_new?checked=2025-01-01T00:00:00Z" \
  -H "Authorization: Bearer <access_token>"
```

---

## OAuth

### GET /oauth/authorize

OAuth 授权页面。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `response_type` | string | 固定为 `code` |
| `client_id` | string | 应用 ID |
| `redirect_uri` | string | 回调地址（可选） |
| `scope` | string | 权限范围（可选） |
| `state` | string | 状态码（可选） |

**响应：**

```json
{
  "code": "<auth_code>",
  "state": "<state>",
  "location": "https://redirect.uri?code=...&state=..."
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/oauth/authorize?response_type=code&client_id=<client_id>&state=xyz" \
  -H "Authorization: Bearer <access_token>"
```

---

### POST /oauth/token

获取 OAuth 访问令牌。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `grant_type` | string | 固定为 `authorization_code` |
| `code` | string | 授权码 |
| `client_id` | string | 应用 ID |
| `client_secret` | string | 应用密钥 |
| `redirect_uri` | string | 回调地址（可选） |

**响应：**

```json
{
  "accessToken": "<access_token>",
  "tokenType": "bearer",
  "expiresIn": 21600,
  "refreshToken": "<refresh_token>"
}
```

**curl 示例：**

```bash
curl -X POST "https://phira.5wyxi.com/oauth/token?grant_type=authorization_code&code=<code>&client_id=<client_id>&client_secret=<secret>"
```

---

### GET /oauth/{id}

获取 OAuth 应用信息。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | string | 应用 client_id |

**响应：** `OAuthAppView`

```json
{
  "id": 123,
  "name": "应用名称",
  "clientId": "<client_id>",
  "redirectUri": "https://...",
  "maxPerm": 0,
  "avatar": "https://...",
  "creator": 123,
  "created": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/oauth/<client_id>
```

---

## 仪表盘

### GET /dash/stables/{id}/profile

获取指定稳定化审核员的档案信息。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `id` | number | 用户 ID |

**响应：**

```json
{
  "userId": 123,
  "userName": "username",
  "actionCount": 100,
  "approvalCount": 80,
  "messageWordCount": 5000,
  "recentAction": [...]
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/dash/stables/123/profile
```

---

### GET /dash/stables/profile-all

获取所有稳定化审核员的档案信息。

**响应：** `StabilizeProfile[]`

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/dash/stables/profile-all
```

---

## 杂项

### POST /upload/{name}

上传文件到服务器。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `name` | string | 文件名 |

**请求体：** 文件二进制内容

**响应：**

```json
{
  "id": "<file_id>",
  "expireAt": "2025-01-01T00:00:00Z"
}
```

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/upload/chart.zip \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/octet-stream" \
  --data-binary @chart.zip
```

---

### GET /files/{*name}

获取文件（通过文件 ID 或路径）。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `name` | string | 文件 ID 或路径 |

**响应：** 302 重定向到 CDN 地址

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/files/<file_id> \
  -H "Authorization: Bearer <access_token>" \
  -L
```

---

### GET /anys/{*name}

获取 anys 协议文件。

**路径参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `name` | string | 文件名 |

**响应：** 302 重定向到 `anys://` 协议地址

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/anys/<filename> -L
```

---

### GET /check-update

检查客户端更新。

**查询参数：**

| 参数 | 类型 | 说明 |
|------|------|------|
| `version` | string | 当前版本号，如 `1.0.0` |
| `flavor` | string | 客户端类型 |

**响应：**

```json
{
  "version": "1.1.0",
  "date": "2025-01-01",
  "description": "更新描述",
  "url": "https://..."
}
```

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/check-update?version=1.0.0&flavor=android"
```

---

### GET /font-bold

获取粗体字体文件。

**查询参数：**

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `cksum` | string | - | 当前字体校验和 |
| `new_bold_font` | boolean | false | 是否使用新字体 |

**响应：** 302 重定向到字体文件

**curl 示例：**

```bash
curl -X GET "https://phira.5wyxi.com/font-bold?cksum=abc123&new_bold_font=false" \
  -H "Authorization: Bearer <access_token>" \
  -L
```

---

### GET /staff

获取 Staff 团队成员列表。

**响应：**

```json
{
  "admins": [1, 2],
  "headReviewers": [3],
  "reviewers": [4, 5],
  "headSupervisors": [6],
  "supervisors": [7, 8]
}
```

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/staff
```

---

### POST /censor-detail

获取文本审查详情（需要审核权限）。

**请求体：**

```json
{
  "text": "待审查文本"
}
```

**响应：** `CensorBlock[]`

**curl 示例：**

```bash
curl -X POST https://phira.5wyxi.com/censor-detail \
  -H "Authorization: Bearer <access_token>" \
  -H "Content-Type: application/json" \
  -d '{"text":"待审查文本"}'
```

---

### GET /terms

获取服务条款文本。

**响应：** 纯文本服务条款

**curl 示例：**

```bash
curl -X GET https://phira.5wyxi.com/terms/zh-CN.txt
```

---
