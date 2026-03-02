//! 内容过滤工具
//!
//! 基于屏蔽词列表过滤漫画，支持繁简转换和多维度匹配

use picacg_api::models::{Category, Comic};
use picacg_config::AppSettings;
use zhconv::{Variant, zhconv};

/// 过滤配置
pub struct FilterConfig<'a> {
    /// 屏蔽词列表
    pub blocked_keywords: &'a [String],
    /// 是否按分类屏蔽
    pub filter_by_category: bool,
    /// 是否按标签屏蔽
    pub filter_by_tag: bool,
    /// 是否按标题屏蔽
    pub filter_by_title: bool,
}

/// 将文本标准化：繁体转简体 + 转小写
fn normalize(text: &str) -> String {
    zhconv(text, Variant::ZhHans).to_lowercase()
}

/// 检查漫画是否应被屏蔽
///
/// 返回 `true` 表示该漫画应被屏蔽（不显示）
pub fn should_block_comic(comic: &Comic, config: &FilterConfig) -> bool {
    if config.blocked_keywords.is_empty() {
        return false;
    }

    // 无任何过滤维度开启时不屏蔽
    if !config.filter_by_category && !config.filter_by_tag && !config.filter_by_title {
        return false;
    }

    for keyword in config.blocked_keywords {
        let normalized_keyword = normalize(keyword);
        if normalized_keyword.is_empty() {
            continue;
        }

        // 分类：精确匹配
        if config.filter_by_category {
            for category in &comic.categories {
                if normalize(category) == normalized_keyword {
                    return true;
                }
            }
        }

        // 标签：精确匹配
        if config.filter_by_tag {
            for tag in &comic.tags {
                if normalize(tag) == normalized_keyword {
                    return true;
                }
            }
        }

        // 标题：子串匹配
        if config.filter_by_title {
            let normalized_title = normalize(&comic.title);
            if normalized_title.contains(&normalized_keyword) {
                return true;
            }
        }
    }

    false
}

/// 检查分类是否应被屏蔽
///
/// 返回 `true` 表示该分类应被屏蔽（不显示）
pub fn should_block_category(category: &Category, config: &FilterConfig) -> bool {
    if config.blocked_keywords.is_empty() || !config.filter_by_category {
        return false;
    }

    let normalized_title = normalize(&category.title);
    for keyword in config.blocked_keywords {
        let normalized_keyword = normalize(keyword);
        if !normalized_keyword.is_empty() && normalized_title == normalized_keyword {
            return true;
        }
    }

    false
}

/// 从 AppSettings 全局配置构建 FilterConfig
///
/// 调用方需要持有返回的 `Vec<String>` 以确保生命周期正确
pub fn load_filter_keywords() -> Vec<String> {
    AppSettings::global().read().filter.blocked_keywords.clone()
}

/// 从 AppSettings 全局配置构建过滤参数
pub fn load_filter_flags() -> (bool, bool, bool) {
    let settings = AppSettings::global().read();
    (
        settings.filter.filter_by_category,
        settings.filter.filter_by_tag,
        settings.filter.filter_by_title,
    )
}

/// 从漫画列表中筛选出未被屏蔽的索引列表
///
/// 返回原始列表中未被屏蔽的漫画索引
pub fn filter_comic_indices(comics: &[Comic], config: &FilterConfig) -> Vec<usize> {
    comics
        .iter()
        .enumerate()
        .filter(|(_, comic)| !should_block_comic(comic, config))
        .map(|(i, _)| i)
        .collect()
}

/// 从分类列表中筛选出未被屏蔽的索引列表
pub fn filter_category_indices(categories: &[Category], config: &FilterConfig) -> Vec<usize> {
    categories
        .iter()
        .enumerate()
        .filter(|(_, cat)| !should_block_category(cat, config))
        .map(|(i, _)| i)
        .collect()
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
        let config = FilterConfig {
            blocked_keywords: &[],
            filter_by_category: true,
            filter_by_tag: true,
            filter_by_title: true,
        };
        assert!(!should_block_comic(&comic, &config));
    }

    #[test]
    fn test_no_filter_dimension_no_block() {
        let comic = make_comic("测试漫画", vec!["分类A"], vec!["标签B"]);
        let config = FilterConfig {
            blocked_keywords: &["分类A".to_string()],
            filter_by_category: false,
            filter_by_tag: false,
            filter_by_title: false,
        };
        assert!(!should_block_comic(&comic, &config));
    }

    #[test]
    fn test_category_exact_match() {
        let comic = make_comic("测试漫画", vec!["禁漫天堂"], vec![]);
        let config = FilterConfig {
            blocked_keywords: &["禁漫天堂".to_string()],
            filter_by_category: true,
            filter_by_tag: false,
            filter_by_title: false,
        };
        assert!(should_block_comic(&comic, &config));
    }

    #[test]
    fn test_tag_exact_match() {
        let comic = make_comic("测试漫画", vec![], vec!["NTR"]);
        let config = FilterConfig {
            blocked_keywords: &["ntr".to_string()],
            filter_by_category: false,
            filter_by_tag: true,
            filter_by_title: false,
        };
        assert!(should_block_comic(&comic, &config));
    }

    #[test]
    fn test_title_substring_match() {
        let comic = make_comic("这是一个测试标题漫画", vec![], vec![]);
        let config = FilterConfig {
            blocked_keywords: &["测试标题".to_string()],
            filter_by_category: false,
            filter_by_tag: false,
            filter_by_title: true,
        };
        assert!(should_block_comic(&comic, &config));
    }

    #[test]
    fn test_traditional_simplified_conversion() {
        // 繁体分类应被简体屏蔽词匹配
        let comic = make_comic("測試漫畫", vec!["禁漫天堂"], vec![]);
        let config = FilterConfig {
            blocked_keywords: &["禁漫天堂".to_string()],
            filter_by_category: true,
            filter_by_tag: false,
            filter_by_title: false,
        };
        assert!(should_block_comic(&comic, &config));
    }

    #[test]
    fn test_filter_comic_indices() {
        let comics = vec![
            make_comic("漫画A", vec!["分类X"], vec![]),
            make_comic("漫画B", vec!["禁漫天堂"], vec![]),
            make_comic("漫画C", vec!["分类Y"], vec![]),
        ];
        let config = FilterConfig {
            blocked_keywords: &["禁漫天堂".to_string()],
            filter_by_category: true,
            filter_by_tag: false,
            filter_by_title: false,
        };
        let indices = filter_comic_indices(&comics, &config);
        assert_eq!(indices, vec![0, 2]);
    }
}
