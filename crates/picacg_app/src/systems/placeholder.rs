//! 占位页面系统
//!
//! 为尚未实现的页面提供占位 UI

use bevy::prelude::*;

use crate::systems::login::AppColors;

// 注意：HomeRoot、setup_home_ui、cleanup_home_ui 已移至 home.rs
// 注意：FavoritesRoot 已移至 favorites.rs

/// 通用占位页面场景（保留供未来可能需要的页面使用）
///
/// 用法：`commands.spawn_scene(placeholder_page(MyRoot, "标题", "🚧",
/// "描述")).insert(ChildOf(content_area));`
#[allow(dead_code)]
fn placeholder_page<R: Component + Default + Clone + Unpin>(
    root_component: R,
    title: &str,
    icon: &str,
    description: &str,
) -> impl Scene + use<R> {
    let title = title.to_string();
    let icon = icon.to_string();
    let description = description.to_string();

    bsn! {
        template_value(root_component)
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
        }
        BackgroundColor(AppColors::BACKGROUND)
        Children [
            (
                // 图标
                Text({icon})
                TextFont { font_size: FontSize::Px(64.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 标题
                Text({title})
                TextFont { font_size: FontSize::Px(24.0) }
                TextColor(AppColors::TEXT)
            ),
            (
                // 描述
                Text({description})
                TextFont { font_size: FontSize::Px(14.0) }
                TextColor(AppColors::TEXT_SECONDARY)
            ),
            (
                // 开发中提示
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    padding: UiRect::new(Val::Px(16.0), Val::Px(16.0), Val::Px(8.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                }
                BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.5))
                template_value(BorderColor::all(AppColors::PRIMARY))
                Children [
                    (
                        Text("🚧 功能开发中...")
                        TextFont { font_size: FontSize::Px(12.0) }
                        TextColor(AppColors::PRIMARY)
                    )
                ]
            ),
        ]
    }
}
