//! 内容过滤工具
//!
//! 基于屏蔽词列表过滤漫画，支持繁简转换和多维度匹配。
//!
//! ## 性能设计
//!
//! `CompiledFilter` 在构造时一次性完成关键词标准化（繁→简 + 小写），
//! 匹配时每个字段只标准化一次、与全部关键词比较（循环反转）。
//! 相比旧实现（每部漫画 × 每个关键词 × 每个字段 重复 zhconv），
//! 分配量从 `漫画数×词数×字段数` 降到 `漫画数×字段数 + 词数`。
//!
//! 调用方应在确认需要过滤时才构造（惰性），不要放在每帧必经路径上。

use picacg_api::models::{Category, Comic};
use picacg_config::AppSettings;
use zhconv::{Variant, zhconv};

/// 将文本标准化：繁体转简体 + 转小写
fn normalize(text: &str) -> String {
    zhconv(text, Variant::ZhHans).to_lowercase()
}

/// 编译后的过滤器：关键词已标准化，可跨多次匹配复用
pub struct CompiledFilter {
    /// 标准化后的屏蔽词（已剔除空词）
    keywords: Vec<String>,
    /// 是否按分类屏蔽
    filter_by_category: bool,
    /// 是否按标签屏蔽
    filter_by_tag: bool,
    /// 是否按标题屏蔽
    filter_by_title: bool,
}

impl CompiledFilter {
    /// 从给定关键词与维度开关构建（测试用；出现非全局配置场景时移除 cfg）
    #[cfg(test)]
    pub fn from_parts(
        keywords: &[String],
        filter_by_category: bool,
        filter_by_tag: bool,
        filter_by_title: bool,
    ) -> Self {
        Self {
            keywords: keywords
                .iter()
                .map(|k| normalize(k))
                .filter(|k| !k.is_empty())
                .collect(),
            filter_by_category,
            filter_by_tag,
            filter_by_title,
        }
    }

    /// 从全局配置构建（一次读锁 + 一次关键词标准化）
    pub fn from_settings() -> Self {
        let settings = AppSettings::global().read();
        let keywords = settings
            .filter
            .blocked_keywords
            .iter()
            .map(|k| normalize(k))
            .filter(|k| !k.is_empty())
            .collect();
        Self {
            keywords,
            filter_by_category: settings.filter.filter_by_category,
            filter_by_tag: settings.filter.filter_by_tag,
            filter_by_title: settings.filter.filter_by_title,
        }
    }

    /// 过滤器是否为空转（无词或无维度 → 一定不屏蔽任何内容）
    pub fn is_noop(&self) -> bool {
        self.keywords.is_empty()
            || (!self.filter_by_category && !self.filter_by_tag && !self.filter_by_title)
    }

    /// 检查漫画是否应被屏蔽（`true` = 不显示）
    pub fn should_block_comic(&self, comic: &Comic) -> bool {
        if self.is_noop() {
            return false;
        }

        // 分类：精确匹配（每个分类只标准化一次）
        if self.filter_by_category {
            for category in &comic.categories {
                let normalized = normalize(category);
                if self.keywords.contains(&normalized) {
                    return true;
                }
            }
        }

        // 标签：精确匹配
        if self.filter_by_tag {
            for tag in &comic.tags {
                let normalized = normalize(tag);
                if self.keywords.contains(&normalized) {
                    return true;
                }
            }
        }

        // 标题：子串匹配（标题只标准化一次）
        if self.filter_by_title {
            let normalized = normalize(&comic.title);
            if self.keywords.iter().any(|k| normalized.contains(k)) {
                return true;
            }
        }

        false
    }

    /// 检查分类是否应被屏蔽（`true` = 不显示）
    pub fn should_block_category(&self, category: &Category) -> bool {
        if self.keywords.is_empty() || !self.filter_by_category {
            return false;
        }
        let normalized = normalize(&category.title);
        self.keywords.contains(&normalized)
    }

    /// 从漫画列表中筛选出未被屏蔽的索引列表
    pub fn filter_comic_indices(&self, comics: &[Comic]) -> Vec<usize> {
        if self.is_noop() {
            return (0..comics.len()).collect();
        }
        comics
            .iter()
            .enumerate()
            .filter(|(_, comic)| !self.should_block_comic(comic))
            .map(|(i, _)| i)
            .collect()
    }

    /// 从分类列表中筛选出未被屏蔽的索引列表
    pub fn filter_category_indices(&self, categories: &[Category]) -> Vec<usize> {
        categories
            .iter()
            .enumerate()
            .filter(|(_, cat)| !self.should_block_category(cat))
            .map(|(i, _)| i)
            .collect()
    }

    /// 保留未被屏蔽的漫画（只克隆保留项，避免整表深拷贝再过滤）
    pub fn filter_comics_cloned(&self, comics: &[Comic]) -> Vec<Comic> {
        comics
            .iter()
            .filter(|c| !self.should_block_comic(c))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use picacg_api::models::ImageInfo;

    use super::*;

    fn make_comic(title: &str, categories: Vec<&str>, tags: Vec<&str>) -> Comic {
        Comic {
            id: "test".to_string(),
            title: title.to_string(),
            author: String::new(),
            pages_count: 0,
            eps_count: 0,
            finished: false,
            categories: categories.into_iter().map(String::from).collect(),
            tags: tags.into_iter().map(String::from).collect(),
            thumb: ImageInfo {
                original_name: String::new(),
                path: String::new(),
                file_server: String::new(),
            },
            likes_count: 0,
            views_count: 0,
            comments_count: 0,
            description: None,
            chinese_team: None,
            created_at: None,
            updated_at: None,
            allow_download: false,
            allow_comment: None,
            is_favourite: None,
            is_liked: None,
        }
    }

    #[test]
    fn test_empty_keywords_no_block() {
        let comic = make_comic("测试漫画", vec!["分类A"], vec!["标签B"]);
        let filter = CompiledFilter::from_parts(&[], true, true, true);
        assert!(!filter.should_block_comic(&comic));
    }

    #[test]
    fn test_no_filter_dimension_no_block() {
        let comic = make_comic("测试漫画", vec!["分类A"], vec!["标签B"]);
        let filter = CompiledFilter::from_parts(&["分类A".to_string()], false, false, false);
        assert!(!filter.should_block_comic(&comic));
    }

    #[test]
    fn test_category_exact_match() {
        let comic = make_comic("测试漫画", vec!["禁漫天堂"], vec![]);
        let filter = CompiledFilter::from_parts(&["禁漫天堂".to_string()], true, false, false);
        assert!(filter.should_block_comic(&comic));
    }

    #[test]
    fn test_tag_exact_match() {
        let comic = make_comic("测试漫画", vec![], vec!["NTR"]);
        let filter = CompiledFilter::from_parts(&["ntr".to_string()], false, true, false);
        assert!(filter.should_block_comic(&comic));
    }

    #[test]
    fn test_title_substring_match() {
        let comic = make_comic("这是一个测试标题漫画", vec![], vec![]);
        let filter = CompiledFilter::from_parts(&["测试标题".to_string()], false, false, true);
        assert!(filter.should_block_comic(&comic));
    }

    #[test]
    fn test_traditional_simplified_conversion() {
        // 繁体分类应被简体屏蔽词匹配
        let comic = make_comic("測試漫畫", vec!["禁漫天堂"], vec![]);
        let filter = CompiledFilter::from_parts(&["禁漫天堂".to_string()], true, false, false);
        assert!(filter.should_block_comic(&comic));
    }

    #[test]
    fn test_filter_comic_indices() {
        let comics = vec![
            make_comic("漫画A", vec!["分类X"], vec![]),
            make_comic("漫画B", vec!["禁漫天堂"], vec![]),
            make_comic("漫画C", vec!["分类Y"], vec![]),
        ];
        let filter = CompiledFilter::from_parts(&["禁漫天堂".to_string()], true, false, false);
        let indices = filter.filter_comic_indices(&comics);
        assert_eq!(indices, vec![0, 2]);
    }
}
