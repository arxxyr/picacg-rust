//! 占位页面系统
//!
//! 为尚未实现的页面提供占位 UI

use bevy::prelude::*;

use crate::{
    components::ContentArea,
    systems::login::{AppColors, FONT_PATH},
};

/// 首页根标记
#[derive(Component)]
pub struct HomeRoot;

// 注意：FavoritesRoot 已移至 favorites.rs

/// 创建通用占位页面
fn spawn_placeholder_page(
    commands: &mut Commands,
    content_area: Entity,
    font: Handle<Font>,
    root_component: impl Component,
    title: &str,
    icon: &str,
    description: &str,
) {
    commands.entity(content_area).with_children(|parent| {
        parent
            .spawn((
                root_component,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(AppColors::BACKGROUND),
            ))
            .with_children(|root| {
                // 图标
                root.spawn((
                    Text::new(icon),
                    TextFont {
                        font: font.clone(),
                        font_size: 64.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                // 标题
                root.spawn((
                    Text::new(title),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT),
                ));

                // 描述
                root.spawn((
                    Text::new(description),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(AppColors::TEXT_SECONDARY),
                ));

                // 开发中提示
                root.spawn((
                    Node {
                        margin: UiRect::top(Val::Px(30.0)),
                        padding: UiRect::new(
                            Val::Px(16.0),
                            Val::Px(16.0),
                            Val::Px(8.0),
                            Val::Px(8.0),
                        ),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.5)),
                    BorderColor::all(AppColors::PRIMARY),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|badge| {
                    badge.spawn((
                        Text::new("🚧 功能开发中..."),
                        TextFont {
                            font: font.clone(),
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(AppColors::PRIMARY),
                    ));
                });
            });
    });
}

// ==================== 首页 ====================

pub fn setup_home_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    content_area_query: Query<Entity, With<ContentArea>>,
) {
    let font: Handle<Font> = asset_server.load(FONT_PATH);

    let content_area = match content_area_query.iter().next() {
        Some(entity) => entity,
        None => {
            tracing::warn!("首页：找不到内容区域");
            return;
        }
    };

    spawn_placeholder_page(
        &mut commands,
        content_area,
        font,
        HomeRoot,
        "首页",
        "🏠",
        "推荐漫画、最新更新、热门作品",
    );

    tracing::info!("首页 UI 已创建");
}

pub fn cleanup_home_ui(mut commands: Commands, query: Query<Entity, With<HomeRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// 注意：收藏页面已移至 favorites.rs
