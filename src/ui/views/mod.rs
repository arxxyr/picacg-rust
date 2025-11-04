pub mod categories;
pub mod comic_detail;
pub mod comics_list;
pub mod home;
pub mod login;
pub mod main_layout;
pub mod proxy_settings;

pub use categories::view as categories_view;
pub use comic_detail::view as comic_detail_view;
pub use comics_list::view as comics_list_view;
pub use home::view as home_view;
pub use login::view as login_view;
pub use main_layout::view as main_layout_view;
pub use proxy_settings::view as proxy_settings_view;
