//! 通用文本输入框组件与系统
//!
//! 提供光标定位、方向键移动、IME 中文输入、剪贴板操作等功能。
//! 页面只需 spawn TextInput + TextInputDisplay，编辑逻辑自动生效。

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    window::{Ime, PrimaryWindow},
};

use crate::systems::login::AppColors;

/// 等宽字符宽度估算（SarasaTermSCNerd, font_size 14.0）
const MONO_CHAR_WIDTH: f32 = 8.4;

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
/// 页面负责 spawn 和焦点管理（设置 `focused`），编辑逻辑由通用系统处理。
#[derive(Component)]
pub struct TextInput {
    /// 当前文本
    pub value: String,
    /// 光标位置（字符索引）
    pub cursor: usize,
    /// 是否获得焦点
    pub focused: bool,
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
            focused: false,
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
#[derive(Component)]
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

// ==================== 系统 ====================

/// 键盘编辑系统 —— 处理字符输入、删除、方向键、剪贴板
pub fn text_input_keyboard(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut input_query: Query<&mut TextInput>,
    key_input: Res<ButtonInput<KeyCode>>,
) {
    let has_focus = input_query.iter().any(|i| i.focused);
    if !has_focus {
        return;
    }

    let ctrl = key_input.pressed(KeyCode::ControlLeft)
        || key_input.pressed(KeyCode::ControlRight)
        || key_input.pressed(KeyCode::SuperLeft)
        || key_input.pressed(KeyCode::SuperRight);

    for event in keyboard_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        for mut input in input_query.iter_mut() {
            if !input.focused {
                continue;
            }

            match &event.logical_key {
                Key::Backspace => {
                    if input.cursor > 0 {
                        let start = char_to_byte_index(&input.value, input.cursor - 1);
                        let end = char_to_byte_index(&input.value, input.cursor);
                        input.value.replace_range(start..end, "");
                        input.cursor -= 1;
                    }
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
}

/// Ctrl/Cmd 快捷键处理
fn handle_ctrl_shortcut(input: &mut Mut<'_, TextInput>, ch: &str) {
    match ch.to_ascii_lowercase().as_str() {
        "v" => {
            // 粘贴
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                    let byte_idx = char_to_byte_index(&input.value, input.cursor);
                    input.value.insert_str(byte_idx, &filtered);
                    input.cursor += filtered.chars().count();
                }
            }
        }
        "c" => {
            // 复制
            if !input.value.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&input.value);
                }
            }
        }
        "x" => {
            // 剪切
            if !input.value.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&input.value);
                }
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

/// IME 中文输入处理
pub fn text_input_ime(mut ime_events: MessageReader<Ime>, mut input_query: Query<&mut TextInput>) {
    for event in ime_events.read() {
        if let Ime::Commit { value, .. } = event {
            for mut input in input_query.iter_mut() {
                if input.focused {
                    let byte_idx = char_to_byte_index(&input.value, input.cursor);
                    input.value.insert_str(byte_idx, value);
                    input.cursor += value.chars().count();
                }
            }
        }
    }
}

/// 点击定位光标（仅更新已聚焦输入框的光标位置）
pub fn text_input_click_position(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut TextInput,
            &GlobalTransform,
            &ComputedNode,
        ),
        Changed<Interaction>,
    >,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for (interaction, mut input, transform, computed) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let text_len = input.value.chars().count();
        let cursor_pos = window_query
            .single()
            .ok()
            .and_then(|window| {
                let cursor = window.cursor_position()?;
                let scale = window.scale_factor();
                let node_w = computed.size().x / scale;
                let node_cx = transform.translation().x;
                let node_left = node_cx - node_w / 2.0;
                let click_x = cursor.x - window.width() / 2.0;
                let relative_x = (click_x - node_left - 12.0).max(0.0);
                let char_pos = (relative_x / MONO_CHAR_WIDTH).round() as usize;
                Some(char_pos.min(text_len))
            })
            .unwrap_or(text_len);

        input.cursor = cursor_pos;
    }
}

/// 光标闪烁 + 文本渲染系统
pub fn text_input_cursor_blink(
    time: Res<Time>,
    mut blink: ResMut<TextInputCursorBlink>,
    input_query: Query<(&TextInput, &Children)>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<TextInputDisplay>>,
) {
    let prev_visible = blink.visible;
    blink.timer.tick(time.delta());
    if blink.timer.just_finished() {
        blink.visible = !blink.visible;
    }

    let blink_changed = prev_visible != blink.visible;

    for (input, children) in input_query.iter() {
        for child in children.iter() {
            let Ok((mut text, mut color)) = text_query.get_mut(child) else {
                continue;
            };

            let display = input.display_value();

            if display.is_empty() && !input.focused {
                // 无焦点且为空 → 占位符
                let new_text = input.placeholder.clone();
                if **text != new_text {
                    **text = new_text;
                    *color = TextColor(AppColors::TEXT_SECONDARY);
                }
            } else if input.focused {
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
