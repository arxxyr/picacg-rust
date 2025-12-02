# PicACG Rust 开发文档

## 目录结构

```
docs/
├── README.md                    # 本文件 - 文档索引
├── progress.md                  # 📊 项目进度报告（主文档）
├── 00_architecture.md           # 🏗️ 系统架构设计
├── 01_api_protocol.md           # 🔌 API 协议文档
│
├── sessions/                    # 📝 会话记录（按日期）
│   └── (会话记录文件)
│
└── archive/                     # 📦 归档文档
    └── (旧版本/历史文档)
```

## 核心文档

| 文档 | 说明 |
|------|------|
| [progress.md](progress.md) | 项目进度报告，包含完成功能、技术债务、里程碑 |
| [00_architecture.md](00_architecture.md) | 系统架构设计，模块划分 |
| [01_api_protocol.md](01_api_protocol.md) | PicACG API 协议文档 |

## 会话记录

按时间倒序排列的开发会话记录：

| 日期 | 主题 | 文件 |
|------|------|------|
| 2025-12-02/03 | 滚动条修复 + 下载顺序修复 | [session_2025-12-02_scrollbar_fix.md](session_2025-12-02_scrollbar_fix.md) |
| 2025-12-01 | Bevy 0.17 迁移 | [session_2025-12-01_bevy_migration.md](session_2025-12-01_bevy_migration.md) |
| 2025-11-06 | 下载功能修复 | [download_bugfix_2025-11-06.md](download_bugfix_2025-11-06.md) |
| 2025-11-05 | API 响应修复 | [bugfix_2025-11-05_api_response.md](bugfix_2025-11-05_api_response.md) |

## 快速链接

- **项目 CLAUDE.md**: 开发规范和常见陷阱
- **Bevy 迁移计划**: [bevy_migration_plan.md](bevy_migration_plan.md)
- **下载功能实现**: [download_feature_implementation.md](download_feature_implementation.md)

## 经验教训摘要

### Bevy 0.17 常见陷阱

1. **Query 组件缺失**: UI 节点需要显式添加 `Transform` 才能获得 `GlobalTransform`
2. **DPI 缩放**: `ComputedNode::size()` 返回物理像素，需除以 `scale_factor`
3. **坐标系转换**: 屏幕坐标 Y 轴向下，Bevy UI 坐标 Y 轴向上

### API 相关

1. **签名算法**: 必须包含完整 URL（含查询参数）
2. **响应结构**: 分页字段是扁平的，不是嵌套的 `PageInfo`
3. **字段可选性**: 列表接口不返回 `description` 等详情字段
4. **数据顺序**: API 返回的列表顺序不可信，需要显式排序
