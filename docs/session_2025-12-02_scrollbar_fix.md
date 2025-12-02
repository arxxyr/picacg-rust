# 会话记录: 滚动条轨道点击修复

**日期**: 2025-12-02

## 问题描述

滚动条轨道点击功能完全没有响应。

### 症状

- 点击滚动条轨道没有任何反应
- 滑块拖拽功能正常
- `debug_track_interaction` 能检测到 `Pressed` 状态
- 但 `scrollbar_track_click` 没有任何输出

## 诊断过程

### 1. 添加调试日志

在 `scrollbar_track_click` 开头添加日志：

```rust
info!(
    "[滚动条] scrollbar_track_click: 开始处理，轨道数量={}",
    track_query.iter().count()
);
```

### 2. 发现问题

日志输出：
```
[滚动条] scrollbar_track_click: 开始处理，轨道数量=0
```

Query 返回 0 个实体！

### 3. 对比分析

对比两个系统的 Query：

**`debug_track_interaction` (工作正常)**:
```rust
track_query: Query<(&ScrollbarTrack, &Interaction, &ComputedNode), Changed<Interaction>>
```

**`scrollbar_track_click` (不工作)**:
```rust
track_query: Query<(&ScrollbarTrack, &Interaction, &GlobalTransform, &ComputedNode), Changed<Interaction>>
```

差异：`scrollbar_track_click` 需要 `GlobalTransform` 组件。

## 根本原因

Bevy UI 节点默认**不包含** `Transform`/`GlobalTransform` 组件。

只有显式添加 `Transform` 组件后，Bevy 才会自动添加 `GlobalTransform`。

滚动条轨道创建时没有添加 `Transform`，导致 Query 无法匹配到任何实体。

## 修复方案

在创建滚动条轨道时添加 `Transform::default()`：

```rust
// src/systems/categories.rs 和 src/systems/comics.rs
scrollbar.spawn((
    ScrollbarTrack { scroll_container },
    Button,
    Interaction::default(),
    Node { /* ... */ },
    BackgroundColor(TRACK_COLOR),
    ZIndex(0),
    // 添加 Transform 以获得 GlobalTransform（滚动条点击需要）
    Transform::default(),
));
```

## 修改文件

| 文件 | 修改内容 |
|------|----------|
| `src/systems/categories.rs` | 滚动条轨道添加 `Transform::default()` |
| `src/systems/comics.rs` | 滚动条轨道添加 `Transform::default()` |
| `src/systems/scrollbar.rs` | 清理调试日志，删除 `debug_track_interaction` |
| `src/plugins/ui_plugin.rs` | 移除 `debug_track_interaction` 系统注册 |
| `.github/workflows/ci.yml` | 新增 GitHub CI workflow |

## 经验总结

### Bevy ECS Query 组件缺失陷阱

**问题模式**：
- Query 需要某个组件，但实体没有该组件
- 查询返回 0 个实体
- 系统函数不报错，但完全没有响应

**诊断技巧**：
1. 添加调试日志检查 Query 匹配数量
2. 对比工作正常的 Query 和有问题的 Query
3. 检查实体创建代码，确认是否缺少必要组件

**Bevy 组件依赖关系**：

| 组件 | 自动添加条件 |
|------|-------------|
| `GlobalTransform` | 需要显式添加 `Transform` |
| `ComputedNode` | UI 节点自动获得 |
| `Interaction` | 需要显式添加，配合 `Button` 或 `FocusPolicy` |

## 相关 Commit

- `8f468b3` fix: 修复滚动条轨道点击无响应问题
- `e9f21af` docs: 更新开发文档 - 添加 Query 组件缺失陷阱

## 文档更新

- `CLAUDE.md` - 添加 "常见陷阱" 章节
- `~/.claude/CLAUDE.md` - 添加 "10.3 Bevy ECS Query 组件缺失" 章节

---

# 2025-12-03 更新

## 下载章节顺序修复

### 问题描述

下载漫画时章节顺序是倒序的（从最后一章开始下载），用户希望从第一章开始下载。

### 日志表现

```
第 31 章共 48 张图片
第 31 章下载完成: 成功=0, 跳过=48, 失败=0
第 30 章共 33 张图片
第 30 章下载完成: 成功=0, 跳过=33, 失败=0
第 29 章共 35 张图片
...
```

### 根本原因

API 返回的章节列表 `detail_state.episodes` 本身是倒序的（从大到小），代码直接遍历这个列表进行下载，导致下载顺序也是倒序。

### 修复方案

在 `handle_download_comic` 和 `handle_resume_download` 函数中，对章节列表进行排序：

```rust
// src/plugins/api_plugin.rs

// 新下载
let mut episodes_to_download: Vec<i32> = if event.episodes.is_empty() {
    detail_state.episodes.iter().map(|e| e.order).collect()
} else {
    event.episodes.clone()
};
// 从第一章开始下载（正序）
episodes_to_download.sort();

// 恢复下载
let task_info = download_state.find_task(&comic_id).map(|fsm| {
    let mut episode_orders = fsm.meta.episode_orders.clone();
    // 从第一章开始下载（正序）
    episode_orders.sort();
    // ...
});
```

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `src/plugins/api_plugin.rs:943-944` | 新下载时排序章节列表 |
| `src/plugins/api_plugin.rs:1384-1386` | 恢复下载时排序章节列表 |

### 经验总结

**API 返回数据顺序不可信**：
- 不要假设 API 返回的列表顺序符合预期
- 如果需要特定顺序，应该显式排序
