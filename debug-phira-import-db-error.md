# Phira Import Database Error

Status: [OPEN]

## Symptom

赛事管理页输入 Phira ID 后点击下载，页面提示“赛事操作失败: database internal error”。

## Reproduction

1. 打开赛事管理页。
2. 输入 Phira ID。
3. 点击下载/导入按钮。
4. 记录页面提示和服务端日志。

## Hypotheses

1. Phira 详情响应与当前反序列化模型不一致。
2. Phira 导入成功后写入 `chart_library` 触发数据库错误。
3. 数据库实际结构与当前 `chart_library` 实体不一致。
4. 前端传递的 Phira ID 非法或字段名不匹配。

## Evidence

等待复现。
