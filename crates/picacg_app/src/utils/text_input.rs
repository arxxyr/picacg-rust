//! 通用文本输入框组件与系统
//!
//! 提供光标定位、方向键移动、IME 中文输入、剪贴板操作等功能。
//! 页面只需 spawn `TextInput` + 子节点
//! `TextInputDisplay`，其余全部由通用系统接管：
//!
//! - **焦点单一真相源**：上游 `bevy::input_focus::InputFocus`（随
//!   DefaultPlugins 初始化）。 此前 8 套页面级焦点实现（Focus 资源 + sync 系统
//!   + 影子布尔）全部废除； 页面读焦点一律 `input_focus.get() ==
//!   Some(entity)`。
//! - 点击聚焦 + 光标定位：`text_input_click_focus`
//! - 聚焦视觉（边框）与 IME 开关：`text_input_focus_visuals`
//! - 点击空白失焦：`text_input_blur`
//! - 键盘/IME 编辑按焦点实体**定向分发**（不再全表扫描找 focused 标志）

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    input_focus::{FocusCause, InputFocus},
    prelude::*,
    ui::RelativeCursorPosition,
    window::{Ime, PrimaryWindow},
};

use crate::systems::login::AppColors;

/// 等宽字符宽度估算（SarasaTermSCNerd, font_size 14.0）
const MONO_CHAR_WIDTH: f32 = 8.4;

/// 单字符的水平步进宽度
///
/// 更纱黑体（Sarasa Term）是严格 2:1 半/全角等宽字体：
/// ASCII 半角 = 1 单位，CJK 等非 ASCII 全角 = 2 单位。
fn char_advance(c: char) -> f32 {
    if c.is_ascii() {
        MONO_CHAR_WIDTH
    } else {
        MONO_CHAR_WIDTH * 2.0
    }
}

/// 字符索引 → 字节偏移
fn char_to_byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

// ==================== 组件 ====================

/// 通用文本输入框组件
///
/// 附加到 Button 实体上，配合 `TextInputDisplay` 子节点使用。
/// 焦点不存在组件里——统一由 `InputFocus` 资源仲裁。
#[derive(Component, Default, Clone)]
pub struct TextInput {
    /// 当前文本
    pub value: String,
    /// 光标位置（字符索引）
    pub cursor: usize,
    /// 占位符文本
    pub placeholder: String,
    /// 密码模式
    pub password: bool,
    /// 显示密码明文（仅密码模式有效）
    pub show_password: bool,
}

impl TextInput {
    /// 创建新的文本输入框
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            placeholder: placeholder.into(),
            password: false,
            show_password: false,
        }
    }

    /// 设置初始值（光标移到末尾）
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let v = value.into();
        self.cursor = v.chars().count();
        self.value = v;
        self
    }

    /// 设为密码模式
    pub fn with_password(mut self) -> Self {
        self.password = true;
        self
    }

    /// 获取显示文本（处理密码掩码）
    pub fn display_value(&self) -> String {
        if self.password && !self.show_password && !self.value.is_empty() {
            "*".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }

    /// 设置值并将光标移到末尾
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.chars().count();
    }
}

/// 文本输入框内的文本显示节点标记
///
/// 附加到 TextInput 的子 Text 节点上。
#[derive(Component, Default, Clone)]
pub struct TextInputDisplay;

/// 全局光标闪烁资源
#[derive(Resource)]
pub struct TextInputCursorBlink {
    pub timer: Timer,
    pub visible: bool,
}

impl Default for TextInputCursorBlink {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.53, TimerMode::Repeating),
            visible: true,
        }
    }
}

// ==================== 焦点仲裁 ====================

/// 点击聚焦 + 光标定位
///
/// 点中输入框：设为焦点实体、按点击位置定位光标、开启 IME 并设置候选框位置。
pub fn text_input_click_focus(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut TextInput,
            &RelativeCursorPosition,
            &ComputedNode,
        ),
        Changed<Interaction>,
    >,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for (entity, interaction, mut input, relative_cursor, computed) in interaction_query.iter_mut()
    {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // 聚焦（比较后写，避免无谓的 Changed 触发）
        if input_focus.get() != Some(entity) {
            input_focus.set(entity, FocusCause::Pressed);
        }

        // 光标定位：逐字符累计步进（CJK 全角占 2 单位），取最近的字符边界
        let Ok(mut window) = window_query.single_mut() else {
            continue;
        };
        let scale_factor = window.scale_factor();
        let text_len = input.value.chars().count();
        let display = input.display_value();
        let cursor_pos = relative_cursor
            .normalized
            .map(|n| {
                let node_w = computed.size().x / scale_factor;
                // 12.0 是输入框左侧 padding
                let relative_x = (n.x * node_w - 12.0).max(0.0);
                let mut advance_sum = 0.0_f32;
                let mut char_pos = 0_usize;
                for c in display.chars() {
                    let advance = char_advance(c);
                    if relative_x < advance_sum + advance / 2.0 {
                        break;
                    }
                    advance_sum += advance;
                    char_pos += 1;
                }
                char_pos.min(text_len)
            })
            .unwrap_or(text_len);
        input.cursor = cursor_pos;

        // IME 候选框跟随点击位置
        window.ime_enabled = true;
        if let Some(cursor_position) = window.cursor_position() {
            let input_height = computed.size().y / scale_factor;
            window.ime_position = Vec2::new(
                cursor_position.x,
                cursor_position.y + input_height / 2.0 + 5.0,
            );
        }
    }
}

/// 点击空白处失焦
///
/// 鼠标左键按下且未命中任何输入框 → 清除焦点（IME 由视觉系统随之关闭）。
pub fn text_input_blur(
    mut input_focus: ResMut<InputFocus>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    input_query: Query<&Interaction, With<TextInput>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(focused) = input_focus.get() else {
        return;
    };
    // 焦点在输入框上且本次点击没落在任何输入框 → 失焦
    let clicked_any_input = input_query
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));
    if !clicked_any_input && input_query.contains(focused) {
        input_focus.clear();
    }
}

/// 聚焦视觉与 IME 开关
///
/// `InputFocus` 变化时：统一刷新所有输入框边框（聚焦 PRIMARY / 失焦 BORDER），
/// 并按「焦点是否在输入框上」开关 IME。取代此前各页手写的边框刷新与
/// 仅 2/16 页记得管的 `ime_enabled`。
pub fn text_input_focus_visuals(
    input_focus: Res<InputFocus>,
    mut inputs: Query<(Entity, &mut BorderColor), With<TextInput>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !input_focus.is_changed() {
        return;
    }

    let focused = input_focus.get();
    let mut any_input_focused = false;
    for (entity, mut border) in inputs.iter_mut() {
        let target = if focused == Some(entity) {
            any_input_focused = true;
            AppColors::PRIMARY
        } else {
            AppColors::BORDER
        };
        let target = BorderColor::all(target);
        if *border != target {
            *border = target;
        }
    }

    if !any_input_focused
        && let Ok(mut window) = window_query.single_mut()
        && window.ime_enabled
    {
        window.ime_enabled = false;
    }
}

// ==================== 编辑 ====================

/// 键盘编辑系统 —— 处理字符输入、删除、方向键、剪贴板（按焦点实体定向分发）
pub fn text_input_keyboard(
    input_focus: Res<InputFocus>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut input_query: Query<&mut TextInput>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut input) = input_query.get_mut(focused) else {
        return;
    };

    let ctrl = key_input.pressed(KeyCode::ControlLeft)
        || key_input.pressed(KeyCode::ControlRight)
        || key_input.pressed(KeyCode::SuperLeft)
        || key_input.pressed(KeyCode::SuperRight);

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match &event.logical_key {
            Key::Backspace if input.cursor > 0 => {
                let start = char_to_byte_index(&input.value, input.cursor - 1);
                let end = char_to_byte_index(&input.value, input.cursor);
                input.value.replace_range(start..end, "");
                input.cursor -= 1;
            }
            Key::Delete => {
                let len = input.value.chars().count();
                if input.cursor < len {
                    let start = char_to_byte_index(&input.value, input.cursor);
                    let end = char_to_byte_index(&input.value, input.cursor + 1);
                    input.value.replace_range(start..end, "");
                }
            }
            Key::ArrowLeft => input.cursor = input.cursor.saturating_sub(1),
            Key::ArrowRight => {
                input.cursor = (input.cursor + 1).min(input.value.chars().count());
            }
            Key::Home => input.cursor = 0,
            Key::End => input.cursor = input.value.chars().count(),
            Key::Character(ch) => {
                if ctrl {
                    handle_ctrl_shortcut(&mut input, ch);
                } else {
                    for c in ch.chars() {
                        if !c.is_control() {
                            let byte_idx = char_to_byte_index(&input.value, input.cursor);
                            input.value.insert(byte_idx, c);
                            input.cursor += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Ctrl/Cmd 快捷键处理
fn handle_ctrl_shortcut(input: &mut Mut<'_, TextInput>, ch: &str) {
    match ch.to_ascii_lowercase().as_str() {
        "v" => {
            // 粘贴
            if let Ok(mut clipboard) = arboard::Clipboard::new()
                && let Ok(text) = clipboard.get_text()
            {
                let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                let byte_idx = char_to_byte_index(&input.value, input.cursor);
                input.value.insert_str(byte_idx, &filtered);
                input.cursor += filtered.chars().count();
            }
        }
        "c" => {
            // 复制
            if !input.value.is_empty()
                && let Ok(mut clipboard) = arboard::Clipboard::new()
            {
                let _ = clipboard.set_text(&input.value);
            }
        }
        "x" => {
            // 剪切
            if !input.value.is_empty()
                && let Ok(mut clipboard) = arboard::Clipboard::new()
            {
                let _ = clipboard.set_text(&input.value);
                input.value.clear();
                input.cursor = 0;
            }
        }
        "a" => {
            // 全选（防止 'a' 被输入）
        }
        _ => {}
    }
}

/// IME 中文输入处理（按焦点实体定向分发）
pub fn text_input_ime(
    input_focus: Res<InputFocus>,
    mut ime_events: MessageReader<Ime>,
    mut input_query: Query<&mut TextInput>,
) {
    let Some(focused) = input_focus.get() else {
        return;
    };
    for event in ime_events.read() {
        if let Ime::Commit { value, .. } = event
            && let Ok(mut input) = input_query.get_mut(focused)
        {
            let byte_idx = char_to_byte_index(&input.value, input.cursor);
            input.value.insert_str(byte_idx, value);
            input.cursor += value.chars().count();
        }
    }
}

// ==================== 渲染 ====================

/// 光标闪烁 + 文本渲染系统
pub fn text_input_cursor_blink(
    time: Res<Time>,
    input_focus: Res<InputFocus>,
    mut blink: ResMut<TextInputCursorBlink>,
    input_query: Query<(Entity, &TextInput, &Children)>,
    changed_inputs: Query<(), Changed<TextInput>>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<TextInputDisplay>>,
) {
    let prev_visible = blink.visible;
    blink.timer.tick(time.delta());
    if blink.timer.just_finished() {
        blink.visible = !blink.visible;
    }

    let blink_changed = prev_visible != blink.visible;

    // 无闪烁翻转、无输入内容变化、无焦点变化时零开销
    if !blink_changed && changed_inputs.is_empty() && !input_focus.is_changed() {
        return;
    }

    for (entity, input, children) in input_query.iter() {
        let focused = input_focus.get() == Some(entity);
        for child in children.iter() {
            let Ok((mut text, mut color)) = text_query.get_mut(child) else {
                continue;
            };

            let display = input.display_value();

            if display.is_empty() && !focused {
                // 无焦点且为空 → 占位符
                let new_text = input.placeholder.clone();
                if **text != new_text {
                    **text = new_text;
                    *color = TextColor(AppColors::TEXT_SECONDARY);
                }
            } else if focused {
                // 有焦点 → 显示光标
                let cursor_char = if blink.visible { "|" } else { " " };
                let cursor_pos = input.cursor.min(display.chars().count());
                let byte_idx = display
                    .char_indices()
                    .nth(cursor_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(display.len());
                let (before, after) = display.split_at(byte_idx);
                let new_text = format!("{}{}{}", before, cursor_char, after);
                // 闪烁每帧都不同，直接更新
                if blink_changed || **text != new_text {
                    **text = new_text;
                    *color = TextColor(AppColors::TEXT);
                }
            } else {
                // 无焦点有内容
                if **text != display {
                    **text = display;
                    *color = TextColor(AppColors::TEXT);
                }
            }
        }
    }
}
