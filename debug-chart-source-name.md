# Chart Source Name Migration Error

Status: [OPEN]

## Symptom

服务启动迁移阶段报错：`null value in column "name" of relation "chart_source" violates not-null constraint`。

## Hypotheses

1. 数据库中的 `chart_source.name` 为必填列，而迁移只插入 `source_type`。
2. `chart_source` 实际列名或约束与当前源码假设不一致。
3. 迁移已部分执行，表已存在但初始化数据插入失败。
4. 启动使用的二进制不是最新源码构建结果。

## Evidence

等待迁移结构核对和启动复现。
