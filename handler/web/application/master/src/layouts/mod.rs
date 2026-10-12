//! Application Layouts
//!
//! The shell itself (`MainLayout` + `<Outlet/>`, auth guard, sidebar, topbar) lives in
//! `web_package::layout`; this module only provides the Master Portal configuration:
//! - `master_layout`: `MasterLayout` wrapper passing brand & menu into `MainLayout`
//! - `menu_service` / `menu_context`: load the menu tree from the database (SUPG002)
//! - `master_navigation`: maps the menu tree to sidebar sections

pub mod master_layout;
pub mod master_navigation;
pub mod menu_context;
pub mod menu_service;

pub use master_layout::MasterLayout;
