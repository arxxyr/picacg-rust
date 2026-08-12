//! 通用控件层：按钮样式与统一交互配色
//!
//! 盘点结论（rv-widgets）：全项目 305 处手写的 `Interaction::Hovered/Pressed`
//! 配色分支（134 + 171），每个按钮一份 3-5 行样板。本模块以一个
//! `ButtonStyle` 组件 + 一个全局系统取代它们：按钮声明"我是什么变体"，
//! 配色由系统统一计算。
//!
//! ## 用法（BSN）
//!
//! ```ignore
//! bsn! {
//!     MyButton
//!     Button                                       // 0.19 起 require(Interaction)，无需再裸写
//!     ButtonStyle { variant: ButtonVariant::Primary }
//!     Node { .. }
//!     Children [ ( Text("确定") .. ) ]
//! }
//! ```
//!
//! 页面交互系统随之只保留 `Interaction::Pressed` 的业务分支；
//! 有选中态的按钮（分段/页签）把 `selected` 置真即可豁免 hover 覆盖。

use bevy::prelude::*;

use super::theme::Theme;

/// 按钮变体（决定三态配色）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// 主按钮（primary 系）
    #[default]
    Primary,
    /// 次要按钮（secondary 系）
    Secondary,
    /// 危险按钮（error 系）
    Danger,
    /// 幽灵按钮（默认透明，悬停浮起 surface_hover）
    Ghost,
    /// 卡片（surface 底，悬停 surface_hover，按下 background）
    Card,
    /// 分段/单选项（未选 surface_sunken，悬停 surface_hover；配合 selected 钉选
    /// primary）
    Segment,
}

/// 按钮样式声明：挂上即由全局系统接管 hover/pressed 配色
#[derive(Component, Default, Clone)]
pub struct ButtonStyle {
    /// 变体
    pub variant: ButtonVariant,
    /// 选中态（分段按钮/页签用）：置真时钉在 primary，忽略 hover/pressed
    pub selected: bool,
}

impl ButtonStyle {
    /// 主按钮
    pub fn primary() -> Self {
        Self {
            variant: ButtonVariant::Primary,
            selected: false,
        }
    }

    /// 次要按钮
    pub fn secondary() -> Self {
        Self {
            variant: ButtonVariant::Secondary,
            selected: false,
        }
    }

    /// 危险按钮
    pub fn danger() -> Self {
        Self {
            variant: ButtonVariant::Danger,
            selected: false,
        }
    }

    /// 幽灵按钮
    pub fn ghost() -> Self {
        Self {
            variant: ButtonVariant::Ghost,
            selected: false,
        }
    }

    /// 卡片
    pub fn card() -> Self {
        Self {
            variant: ButtonVariant::Card,
            selected: false,
        }
    }

    /// 分段/单选项（排序/页签/性别/格式等单选组的标准构造器）
    ///
    /// 未选中 = surface_sunken 系；选中钉在 primary。
    /// 注意不要用 `selectable(Primary, ..)` 做单选组——Primary 的静息色
    /// 就是 primary，未选项会与选中项同色（两个 E 波 agent 实战撞出的坑）。
    pub fn segment(selected: bool) -> Self {
        Self {
            variant: ButtonVariant::Segment,
            selected,
        }
    }

    /// 带选中态（特殊场景自定变体；单选组请用 `segment`）
    pub fn selectable(variant: ButtonVariant, selected: bool) -> Self {
        Self { variant, selected }
    }

    /// 按交互状态解析背景色
    fn resolve(&self, interaction: Interaction, theme: &Theme) -> Color {
        if self.selected {
            return theme.primary;
        }
        match self.variant {
            ButtonVariant::Primary => match interaction {
                Interaction::Pressed => theme.primary_pressed,
                Interaction::Hovered => theme.primary_hover,
                Interaction::None => theme.primary,
            },
            ButtonVariant::Secondary => match interaction {
                Interaction::Pressed => theme.secondary,
                Interaction::Hovered => theme.secondary_hover,
                Interaction::None => theme.secondary,
            },
            ButtonVariant::Danger => match interaction {
                Interaction::Pressed => theme.error.darker(0.1),
                Interaction::Hovered => theme.error.lighter(0.05),
                Interaction::None => theme.error.darker(0.05),
            },
            ButtonVariant::Ghost => match interaction {
                Interaction::Pressed => theme.surface_sunken,
                Interaction::Hovered => theme.surface_hover,
                Interaction::None => Color::NONE,
            },
            ButtonVariant::Card => match interaction {
                Interaction::Pressed => theme.background,
                Interaction::Hovered => theme.surface_hover,
                Interaction::None => theme.surface,
            },
            ButtonVariant::Segment => match interaction {
                Interaction::Pressed => theme.surface_sunken,
                Interaction::Hovered => theme.surface_hover,
                Interaction::None => theme.surface_sunken,
            },
        }
    }
}

/// 全局按钮配色系统：所有带 `ButtonStyle` 的按钮三态配色统一在此
///
/// `Changed` 双过滤保证静止零开销；`selected` 变化（页面改写 ButtonStyle）
/// 同样触发重算。
pub fn apply_button_interaction(
    mut buttons: Query<
        (&Interaction, &ButtonStyle, &mut BackgroundColor),
        Or<(Changed<Interaction>, Changed<ButtonStyle>)>,
    >,
) {
    let theme = Theme::dark();
    for (interaction, style, mut color) in &mut buttons {
        let target = style.resolve(*interaction, &theme);
        if color.0 != target {
            color.0 = target;
        }
    }
}
