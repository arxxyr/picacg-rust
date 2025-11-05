# PicACG Rust 客户端开发笔记

## 已知问题与解决方案

### 1. iced 0.13 scrollable 组件崩溃问题

**问题描述：**
点击漫画进入详情页时，应用崩溃并显示错误：
```
thread 'main' panicked at scrollable.rs:118:9:
scrollable content must not fill its vertical scrolling axis
```

**根本原因：**
在 iced 0.13 框架中，有两个关键问题会导致 scrollable 崩溃：

1. **不应该在 `scrollable()` 组件本身上设置 `.width(Length::Fill)`**
2. **scrollable 内部的元素不应使用 `.center_x(Length::Fill)` 或 `.center_y(Length::Fill)`**

这两种情况都会导致内部布局计算错误，使得内容在垂直方向上也尝试填充，违反了 "scrollable content must not fill its vertical scrolling axis" 规则。

**错误代码示例 1：scrollable 上设置 width**
```rust
// ❌ 错误写法
container(scrollable(content).width(Length::Fill))
    .width(Length::Fill)
    .height(Length::Fill)

// ✅ 正确写法
container(scrollable(content))
    .width(Length::Fill)
    .height(Length::Fill)
```

**错误代码示例 2：scrollable 内部元素使用 center_x/center_y**
```rust
// ❌ 错误写法（位于 scrollable 内部）
container(image(handle.clone()))
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(280.0))
    .center_x(Length::Fill)  // 导致崩溃
    .center_y(Length::Fill)  // 导致崩溃

// ✅ 正确写法
container(image(handle.clone()))
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(280.0))
// 固定尺寸的容器会自动居中，无需额外设置
```

**技术原理：**
- `scrollable()` 组件会自动处理其内容的布局
- 当在 scrollable 上直接设置 `.width(Length::Fill)` 时，会导致内部布局计算出现问题
- scrollable 内部使用 `center_x(Length::Fill)` 或 `center_y(Length::Fill)` 会让布局引擎误认为容器想要填充整个垂直滚动区域
- 正确做法是让外层的 `container` 来控制宽度和高度，scrollable 只负责内容的滚动
- 固定尺寸的容器会自然地在父容器中布局，无需显式居中

**已修复的文件：**
- `src/ui/views/comic_detail.rs:215` - 移除 scrollable 的 `.width(Length::Fill)`
- `src/ui/views/comic_detail.rs:82,92` - 移除 cover_widget 的 `.center(Length::Fill)`
- `src/ui/views/comics_list.rs:176` - 移除 scrollable 的 `.width(Length::Fill)`
- `src/ui/views/comics_list.rs:96-97` - 移除 placeholder 的 `.center_x/.center_y(Length::Fill)`
- `src/ui/views/categories.rs:124` - 移除 scrollable 的 `.width(Length::Fill)`
- `src/ui/views/categories.rs:93-94` - 移除 placeholder 的 `.center_x/.center_y(Length::Fill)`
- `src/ui/views/read_view.rs:158-159` - 移除 scaled_container 的 `.center_x/.center_y(Length::Fill)` (2025-11-05)

**提交信息：**
```
fix: 修复 iced scrollable 组件布局导致的崩溃问题

问题描述：
- 点击漫画进入详情页时应用崩溃
- 错误信息：scrollable content must not fill its vertical scrolling axis
- 崩溃位置：comic_detail.rs:215, comics_list.rs:96, categories.rs:93

修复内容：
1. 移除 scrollable 组件上的 .width(Length::Fill) 调用
   - 改为由外层 container 控制宽度和高度
   - 让 scrollable 组件自动处理其内容布局

2. 移除 scrollable 内部元素的 center_x/center_y(Length::Fill)
   - cover_widget 容器移除 .center(Length::Fill)
   - placeholder 容器移除 .center_x/.center_y(Length::Fill)
   - 固定尺寸的容器会自然居中，无需额外设置

业务逻辑：
- iced 0.13 框架中，scrollable 组件不应直接设置 width/height
- scrollable 内部的元素不应使用 Length::Fill 作为居中参数
- 应该通过外层 container 来控制可滚动区域的尺寸
- scrollable 内部会自动根据内容计算滚动区域

影响文件：
- src/ui/views/comic_detail.rs (line 82, 92, 215)
- src/ui/views/comics_list.rs (line 96-97, 176)
- src/ui/views/categories.rs (line 93-94, 124)
```

---

### 2. Tab 键导航系统

**功能描述：**
登录界面支持 Tab 键在所有交互元素间循环导航。

**实现要点：**
- 使用 `keyboard::on_key_press` 订阅键盘事件
- 使用 `LoginFocus` 枚举追踪焦点状态
- Tab 键循环顺序：用户名 → 密码 → 登录按钮 → 代理设置按钮 → 用户名
- Enter 键根据当前焦点执行对应操作
- 通过 `button::Style` 为获得焦点的按钮添加蓝色边框视觉反馈

**相关文件：**
- `src/ui/message.rs` - 新增 `TabPressed` 和 `EnterPressed` 消息
- `src/ui/state.rs` - 扩展 `LoginFocus` 枚举至 4 个变体
- `src/ui/app.rs` - 实现 Tab/Enter 键处理逻辑和订阅
- `src/ui/views/login.rs` - 添加焦点视觉反馈

---

## 开发注意事项

### iced 框架最佳实践

1. **避免在 scrollable 上直接设置尺寸**
   ```rust
   // 错误
   scrollable(content).width(Length::Fill)

   // 正确
   container(scrollable(content)).width(Length::Fill)
   ```

2. **避免在 scrollable 内部元素使用 center_x/center_y(Length::Fill)**
   ```rust
   // 错误（位于 scrollable 内部）
   container(widget)
       .width(Length::Fixed(200.0))
       .center_x(Length::Fill)  // 导致崩溃

   // 正确
   container(widget)
       .width(Length::Fixed(200.0))
   // 固定尺寸会自动居中
   ```

3. **使用 Task 而非递归 update() 调用**
   ```rust
   // 错误
   Message::SomeEvent => {
       self.update(Message::AnotherEvent)
   }

   // 正确
   Message::SomeEvent => {
       Task::done(Message::AnotherEvent)
   }
   ```

4. **keyboard subscription 不能捕获变量**
   ```rust
   // 错误
   keyboard::on_key_press(move |key, modifiers| {
       match focus { ... }  // 捕获了 focus 变量
   })

   // 正确
   keyboard::on_key_press(|key, modifiers| {
       Some(Message::TabPressed)  // 只返回消息
   })
   ```

### 调试技巧

1. **使用完整堆栈追踪**
   ```powershell
   $env:RUST_BACKTRACE = "1"
   cargo run
   ```

2. **通过 grep 快速定位问题**
   ```bash
   cargo run 2>&1 | grep -i "panic\|error"
   ```

3. **检查所有 scrollable 使用位置**
   ```bash
   grep -rn "scrollable" src/ui/views/
   ```

---

## 待办事项

- [ ] 清理 151 个编译警告（未使用的导入和变量）
- [ ] 实现漫画详情页的"开始阅读"功能
- [ ] 实现收藏和点赞功能
- [ ] 添加搜索功能
- [ ] 优化图片加载性能

---

## 参考资料

- [iced 官方文档](https://docs.rs/iced/latest/iced/)
- [iced GitHub 仓库](https://github.com/iced-rs/iced)
- [PicACG API 文档](../docs/)
