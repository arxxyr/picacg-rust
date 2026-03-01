//! 滚动条组件定义

use bevy::prelude::*;

/// 轨道点击查询过滤器（用于简化 Query 类型签名）
pub type TrackClickFilter = (Changed<Interaction>, With<ScrollbarTrack>);

/// 滚动条配置
#[derive(Clone, Debug)]
pub struct ScrollbarConfig {
    /// 滚动条宽度
    pub width: f32,
    /// 滑块最小高度
    pub thumb_min_height: f32,
    /// 轨道颜色
    pub track_color: Color,
    /// 滑块默认颜色
    pub thumb_color: Color,
    /// 滑块悬停颜色
    pub thumb_hover_color: Color,
    /// 滑块按下颜色
    pub thumb_pressed_color: Color,
}

impl Default for ScrollbarConfig {
    fn default() -> Self {
        Self {
            width: 12.0,
            thumb_min_height: 30.0,
            track_color: Color::srgba(0.2, 0.2, 0.25, 0.3),
            thumb_color: Color::srgba(0.5, 0.5, 0.55, 0.6),
            thumb_hover_color: Color::srgba(0.6, 0.6, 0.65, 0.8),
            thumb_pressed_color: Color::srgba(0.7, 0.7, 0.75, 0.9),
        }
    }
}

/// 滚动条配置常量（兼容旧代码）
pub mod scrollbar_config {
    use bevy::color::Color;

    /// 滚动条宽度
    pub const SCROLLBAR_WIDTH: f32 = 12.0;
    /// 滑块最小高度
    pub const THUMB_MIN_HEIGHT: f32 = 30.0;
    /// 滚动条轨道颜色（透明）
    pub const TRACK_COLOR: Color = Color::srgba(0.2, 0.2, 0.25, 0.3);
    /// 滑块默认颜色
    pub const THUMB_COLOR: Color = Color::srgba(0.5, 0.5, 0.55, 0.6);
    /// 滑块悬停颜色
    pub const THUMB_HOVER_COLOR: Color = Color::srgba(0.6, 0.6, 0.65, 0.8);
    /// 滑块按下颜色
    pub const THUMB_PRESSED_COLOR: Color = Color::srgba(0.7, 0.7, 0.75, 0.9);
}

/// 滚动条容器（包含轨道和滑块）
#[derive(Component)]
pub struct ScrollbarContainer {
    /// 关联的滚动容器实体
    pub scroll_container: Entity,
}

/// 滚动条轨道（可点击跳转）
#[derive(Component)]
pub struct ScrollbarTrack {
    /// 关联的滚动容器实体
    pub scroll_container: Entity,
}

/// 滚动条滑块（可拖拽）
#[derive(Component)]
pub struct ScrollbarThumb {
    /// 关联的滚动容器实体
    pub scroll_container: Entity,
}

/// 滚动条拖拽状态
#[derive(Resource, Default)]
pub struct ScrollbarDragState {
    /// 是否正在拖拽
    pub is_dragging: bool,
    /// 正在拖拽的滚动容器实体
    pub dragging_thumb: Option<Entity>,
    /// 拖拽开始时的鼠标 Y 坐标
    pub drag_start_y: f32,
    /// 拖拽开始时的滚动位置
    pub drag_start_scroll: f32,
}

impl ScrollbarDragState {
    /// 开始拖拽
    pub fn start_drag(&mut self, scroll_container: Entity, mouse_y: f32, scroll_y: f32) {
        self.is_dragging = true;
        self.dragging_thumb = Some(scroll_container);
        self.drag_start_y = mouse_y;
        self.drag_start_scroll = scroll_y;
    }

    /// 结束拖拽
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.dragging_thumb = None;
        self.drag_start_y = 0.0;
        self.drag_start_scroll = 0.0;
    }
}

/// 内容尺寸信息（用于计算滚动条）
#[derive(Component, Default, Clone, Copy)]
pub struct ContentSizeInfo {
    /// 内容总高度
    pub content_height: f32,
    /// 可视区域高度
    pub viewport_height: f32,
}

/// 网格布局参数（用于计算内容高度）
#[derive(Clone, Copy, Debug)]
pub struct GridLayoutParams {
    /// 项目数量
    pub item_count: usize,
    /// 每个项目的宽度
    pub item_width: f32,
    /// 每个项目的高度
    pub item_height: f32,
    /// 容器宽度（不含滚动条）
    pub container_width: f32,
    /// 列间距
    pub column_gap: f32,
    /// 行间距
    pub row_gap: f32,
    /// 上内边距
    pub padding_top: f32,
    /// 下内边距
    pub padding_bottom: f32,
}

impl Default for GridLayoutParams {
    fn default() -> Self {
        Self {
            item_count: 0,
            item_width: 100.0,
            item_height: 100.0,
            container_width: 800.0,
            column_gap: 10.0,
            row_gap: 10.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
        }
    }
}

impl ContentSizeInfo {
    /// 计算网格布局的内容高度
    ///
    /// # 参数
    /// - `params`: 网格布局参数
    ///
    /// # 示例
    /// ```ignore
    /// let height = ContentSizeInfo::calculate_grid_content_height(&GridLayoutParams {
    ///     item_count: 100,
    ///     item_width: 150.0,
    ///     item_height: 200.0,
    ///     container_width: 800.0,
    ///     column_gap: 10.0,
    ///     row_gap: 10.0,
    ///     padding_top: 20.0,
    ///     padding_bottom: 20.0,
    /// });
    /// ```
    pub fn calculate_grid_content_height(params: &GridLayoutParams) -> f32 {
        if params.item_count == 0 || params.container_width <= 0.0 {
            return 0.0;
        }

        // 计算每行能放多少个项目
        let items_per_row = ((params.container_width + params.column_gap)
            / (params.item_width + params.column_gap))
            .floor()
            .max(1.0) as usize;

        // 计算需要多少行
        let row_count = params.item_count.div_ceil(items_per_row);

        // 计算内容高度
        if row_count == 0 {
            params.padding_top + params.padding_bottom
        } else {
            params.padding_top
                + (row_count as f32 * params.item_height)
                + ((row_count - 1) as f32 * params.row_gap)
                + params.padding_bottom
        }
    }
}
