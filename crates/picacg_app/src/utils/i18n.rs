//! 多语言支持（i18n）
//!
//! 基于键值对的轻量级翻译系统，支持简体中文、繁体中文、英文。
//! 使用 `I18n` 资源在 Bevy ECS 系统中获取翻译文本。

use std::collections::HashMap;

use bevy::prelude::*;
use picacg_config::Language;

/// 多语言翻译资源
///
/// 在 Bevy 系统中通过 `Res<I18n>` 注入，使用 `i18n.t("key")` 获取翻译。
/// 如果 key 不存在，返回 key 本身作为 fallback。
#[derive(Resource)]
pub struct I18n {
    translations: HashMap<&'static str, &'static str>,
}

impl I18n {
    /// 根据语言创建翻译资源
    pub fn new(language: Language) -> Self {
        let translations = match language {
            Language::ZhCN => Self::zh_cn(),
            Language::ZhTW => Self::zh_tw(),
            Language::En => Self::en(),
        };
        Self { translations }
    }

    /// 获取翻译文本，key 不存在时返回 key 本身
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.translations.get(key).copied().unwrap_or(key)
    }

    /// 简体中文翻译表
    fn zh_cn() -> HashMap<&'static str, &'static str> {
        let mut m = HashMap::new();

        // ==================== 侧边栏 ====================
        m.insert("sidebar.user", "用户");
        m.insert("sidebar.navigation", "导航");
        m.insert("sidebar.other", "其他");
        m.insert("sidebar.favorites", "我的收藏");
        m.insert("sidebar.history", "阅读历史");
        m.insert("sidebar.like_records", "点赞记录");
        m.insert("sidebar.profile", "个人资料");
        m.insert("sidebar.home", "首页");
        m.insert("sidebar.categories", "分类");
        m.insert("sidebar.search", "搜索");
        m.insert("sidebar.rankings", "排行榜");
        m.insert("sidebar.games", "游戏区");
        m.insert("sidebar.fried", "锅贴社区");
        m.insert("sidebar.chat", "聊天室");
        m.insert("sidebar.image_convert", "图片转换");
        m.insert("sidebar.waifu2x", "Waifu2x 超分");
        m.insert("sidebar.nas", "NAS 存储");
        m.insert("sidebar.tools", "工具");
        m.insert("sidebar.local_read", "本地阅读");
        m.insert("sidebar.downloads", "下载");
        m.insert("sidebar.settings", "设置");
        m.insert("sidebar.subtitle", "漫画客户端");

        // ==================== 通用 ====================
        m.insert("common.loading", "加载中...");
        m.insert("common.error", "错误");
        m.insert("common.retry", "重试");
        m.insert("common.back", "返回");
        m.insert("common.save", "保存");
        m.insert("common.cancel", "取消");
        m.insert("common.confirm", "确认");
        m.insert("common.delete", "删除");
        m.insert("common.search", "搜索");
        m.insert("common.no_data", "暂无数据");
        m.insert("common.close", "关闭");
        m.insert("common.refresh", "刷新");
        m.insert("common.add", "添加");

        // ==================== 登录 ====================
        m.insert("login.title", "登录");
        m.insert("login.email", "邮箱 / 用户名");
        m.insert("login.password", "密码");
        m.insert("login.login_button", "登录");
        m.insert("login.register", "注册");
        m.insert("login.forgot_password", "忘记密码");
        m.insert("login.remember_password", "记住密码");
        m.insert("login.auto_login", "自动登录");
        m.insert("login.auto_punch_in", "自动打卡");

        // ==================== 设置 ====================
        m.insert("settings.title", "设置");
        m.insert("settings.proxy", "代理设置");
        m.insert("settings.download", "下载设置");
        m.insert("settings.log", "日志设置");
        m.insert("settings.cache", "缓存设置");
        m.insert("settings.filter", "内容过滤");
        m.insert("settings.channel", "分流设置");
        m.insert("settings.theme", "主题设置");
        m.insert("settings.language", "语言设置");
        m.insert("settings.language.label", "界面语言");
        m.insert(
            "settings.language.hint",
            "切换界面显示语言，修改后需重启应用生效",
        );
        m.insert("settings.advanced", "高级设置");
        m.insert("settings.network_diag", "网络诊断");
        m.insert("settings.about", "关于");
        m.insert("settings.saved", "设置已保存");
        m.insert(
            "settings.language.changed",
            "语言已切换为「{}」，重启应用后生效",
        );

        // ==================== 排行榜 ====================
        m.insert("rankings.daily", "日榜");
        m.insert("rankings.weekly", "周榜");
        m.insert("rankings.monthly", "月榜");
        m.insert("rankings.knight", "骑士榜");

        // ==================== 阅读器 ====================
        m.insert("reader.prev_page", "上一页");
        m.insert("reader.next_page", "下一页");
        m.insert("reader.prev_chapter", "上一章");
        m.insert("reader.next_chapter", "下一章");

        // ==================== 下载 ====================
        m.insert("downloads.downloading", "下载中");
        m.insert("downloads.waiting", "等待中");
        m.insert("downloads.stopped", "已暂停");
        m.insert("downloads.completed", "已完成");
        m.insert("downloads.pause", "暂停");
        m.insert("downloads.resume", "恢复");
        m.insert("downloads.delete", "删除");

        m
    }

    /// 繁体中文翻译表（基于简体翻译覆盖差异项）
    fn zh_tw() -> HashMap<&'static str, &'static str> {
        let mut m = Self::zh_cn();

        // 侧边栏
        m.insert("sidebar.user", "使用者");
        m.insert("sidebar.navigation", "導航");
        m.insert("sidebar.other", "其他");
        m.insert("sidebar.favorites", "我的收藏");
        m.insert("sidebar.history", "閱讀歷史");
        m.insert("sidebar.like_records", "按讚記錄");
        m.insert("sidebar.profile", "個人資料");
        m.insert("sidebar.home", "首頁");
        m.insert("sidebar.categories", "分類");
        m.insert("sidebar.search", "搜尋");
        m.insert("sidebar.rankings", "排行榜");
        m.insert("sidebar.games", "遊戲區");
        m.insert("sidebar.fried", "鍋貼社區");
        m.insert("sidebar.chat", "聊天室");
        m.insert("sidebar.image_convert", "圖片轉換");
        m.insert("sidebar.waifu2x", "Waifu2x 超分");
        m.insert("sidebar.nas", "NAS 儲存");
        m.insert("sidebar.tools", "工具");
        m.insert("sidebar.local_read", "本機閱讀");
        m.insert("sidebar.downloads", "下載");
        m.insert("sidebar.settings", "設定");
        m.insert("sidebar.subtitle", "漫畫客戶端");

        // 通用
        m.insert("common.loading", "載入中...");
        m.insert("common.error", "錯誤");
        m.insert("common.retry", "重試");
        m.insert("common.back", "返回");
        m.insert("common.save", "儲存");
        m.insert("common.cancel", "取消");
        m.insert("common.confirm", "確認");
        m.insert("common.delete", "刪除");
        m.insert("common.search", "搜尋");
        m.insert("common.no_data", "暫無資料");
        m.insert("common.close", "關閉");
        m.insert("common.refresh", "重新整理");
        m.insert("common.add", "新增");

        // 登录
        m.insert("login.title", "登入");
        m.insert("login.email", "信箱 / 使用者名稱");
        m.insert("login.password", "密碼");
        m.insert("login.login_button", "登入");
        m.insert("login.register", "註冊");
        m.insert("login.forgot_password", "忘記密碼");
        m.insert("login.remember_password", "記住密碼");
        m.insert("login.auto_login", "自動登入");
        m.insert("login.auto_punch_in", "自動打卡");

        // 设置
        m.insert("settings.title", "設定");
        m.insert("settings.proxy", "代理設定");
        m.insert("settings.download", "下載設定");
        m.insert("settings.log", "日誌設定");
        m.insert("settings.cache", "快取設定");
        m.insert("settings.filter", "內容過濾");
        m.insert("settings.channel", "分流設定");
        m.insert("settings.theme", "主題設定");
        m.insert("settings.language", "語言設定");
        m.insert("settings.language.label", "介面語言");
        m.insert(
            "settings.language.hint",
            "切換介面顯示語言，修改後需重新啟動應用程式生效",
        );
        m.insert("settings.advanced", "進階設定");
        m.insert("settings.network_diag", "網路診斷");
        m.insert("settings.about", "關於");
        m.insert("settings.saved", "設定已儲存");
        m.insert(
            "settings.language.changed",
            "語言已切換為「{}」，重新啟動應用程式後生效",
        );

        // 排行榜
        m.insert("rankings.daily", "日榜");
        m.insert("rankings.weekly", "週榜");
        m.insert("rankings.monthly", "月榜");
        m.insert("rankings.knight", "騎士榜");

        // 阅读器
        m.insert("reader.prev_page", "上一頁");
        m.insert("reader.next_page", "下一頁");
        m.insert("reader.prev_chapter", "上一章");
        m.insert("reader.next_chapter", "下一章");

        // 下载
        m.insert("downloads.downloading", "下載中");
        m.insert("downloads.waiting", "等待中");
        m.insert("downloads.stopped", "已暫停");
        m.insert("downloads.completed", "已完成");
        m.insert("downloads.pause", "暫停");
        m.insert("downloads.resume", "恢復");
        m.insert("downloads.delete", "刪除");

        m
    }

    /// 英文翻译表
    fn en() -> HashMap<&'static str, &'static str> {
        let mut m = HashMap::new();

        // 侧边栏
        m.insert("sidebar.user", "User");
        m.insert("sidebar.navigation", "Navigation");
        m.insert("sidebar.other", "Other");
        m.insert("sidebar.favorites", "Favorites");
        m.insert("sidebar.history", "History");
        m.insert("sidebar.like_records", "Likes");
        m.insert("sidebar.profile", "Profile");
        m.insert("sidebar.home", "Home");
        m.insert("sidebar.categories", "Categories");
        m.insert("sidebar.search", "Search");
        m.insert("sidebar.rankings", "Rankings");
        m.insert("sidebar.games", "Games");
        m.insert("sidebar.fried", "Fried");
        m.insert("sidebar.chat", "Chat");
        m.insert("sidebar.image_convert", "Convert");
        m.insert("sidebar.waifu2x", "Waifu2x");
        m.insert("sidebar.nas", "NAS");
        m.insert("sidebar.tools", "Tools");
        m.insert("sidebar.local_read", "Local");
        m.insert("sidebar.downloads", "Downloads");
        m.insert("sidebar.settings", "Settings");
        m.insert("sidebar.subtitle", "Comic Client");

        // 通用
        m.insert("common.loading", "Loading...");
        m.insert("common.error", "Error");
        m.insert("common.retry", "Retry");
        m.insert("common.back", "Back");
        m.insert("common.save", "Save");
        m.insert("common.cancel", "Cancel");
        m.insert("common.confirm", "Confirm");
        m.insert("common.delete", "Delete");
        m.insert("common.search", "Search");
        m.insert("common.no_data", "No data");
        m.insert("common.close", "Close");
        m.insert("common.refresh", "Refresh");
        m.insert("common.add", "Add");

        // 登录
        m.insert("login.title", "Sign In");
        m.insert("login.email", "Email / Username");
        m.insert("login.password", "Password");
        m.insert("login.login_button", "Sign In");
        m.insert("login.register", "Register");
        m.insert("login.forgot_password", "Forgot Password");
        m.insert("login.remember_password", "Remember Password");
        m.insert("login.auto_login", "Auto Login");
        m.insert("login.auto_punch_in", "Auto Punch In");

        // 设置
        m.insert("settings.title", "Settings");
        m.insert("settings.proxy", "Proxy");
        m.insert("settings.download", "Download");
        m.insert("settings.log", "Logging");
        m.insert("settings.cache", "Cache");
        m.insert("settings.filter", "Content Filter");
        m.insert("settings.channel", "Channel");
        m.insert("settings.theme", "Theme");
        m.insert("settings.language", "Language");
        m.insert("settings.language.label", "Interface Language");
        m.insert(
            "settings.language.hint",
            "Switch the interface language. Restart the app for changes to take effect.",
        );
        m.insert("settings.advanced", "Advanced");
        m.insert("settings.network_diag", "Network Diagnostics");
        m.insert("settings.about", "About");
        m.insert("settings.saved", "Settings saved");
        m.insert(
            "settings.language.changed",
            "Language switched to \"{}\". Restart the app for changes to take effect.",
        );

        // 排行榜
        m.insert("rankings.daily", "Daily");
        m.insert("rankings.weekly", "Weekly");
        m.insert("rankings.monthly", "Monthly");
        m.insert("rankings.knight", "Knight");

        // 阅读器
        m.insert("reader.prev_page", "Previous");
        m.insert("reader.next_page", "Next");
        m.insert("reader.prev_chapter", "Prev Chapter");
        m.insert("reader.next_chapter", "Next Chapter");

        // 下载
        m.insert("downloads.downloading", "Downloading");
        m.insert("downloads.waiting", "Waiting");
        m.insert("downloads.stopped", "Paused");
        m.insert("downloads.completed", "Completed");
        m.insert("downloads.pause", "Pause");
        m.insert("downloads.resume", "Resume");
        m.insert("downloads.delete", "Delete");

        m
    }
}
