# PicACG Rust 开发文档

## 文档列表

| 文档 | 说明 |
|------|------|
| [progress.md](progress.md) | 📊 项目进度报告（包含开发历史和待办事项） |
| [00_architecture.md](00_architecture.md) | 🏗️ 系统架构设计 |
| [01_api_protocol.md](01_api_protocol.md) | 🔌 PicACG API 协议文档 |

## 快速入口

- **当前框架**: Bevy 0.17.3 (ECS 架构)
- **项目 CLAUDE.md**: 开发规范和常见陷阱
- **进度概览**: 见 [progress.md](progress.md)

## 常见问题速查

### Bevy 0.17 API

| API | 说明 |
|-----|------|
| `#[derive(Message)]` | 定义消息/事件 |
| `MessageWriter::write()` | 发送消息 |
| `MessageReader<T>` | 接收消息 |
| `ScrollPosition.y` | 滚动位置 |
| `despawn()` | 删除实体（自动递归） |

### 常见陷阱

1. **Query 组件缺失**: UI 节点需显式添加 `Transform` 才有 `GlobalTransform`
2. **DPI 缩放**: `ComputedNode::size()` 返回物理像素，需除以 `scale_factor`
3. **坐标系**: 屏幕 Y 向下，Bevy UI Y 向上
4. **API 数据顺序**: 不可信，需显式排序
