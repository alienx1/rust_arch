//! Navigation types shared by `MainSidebar` (menu) and `MainTopbar` (breadcrumb title)
//!
//! The menu is data, not code: an application builds `NavSection`s at runtime (for example from
//! the menu tree its API returns) and hands `MainLayout` a `Signal<Vec<NavSection>>`, so the
//! sidebar follows whatever the database says.

/// One sidebar link
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub href: String,
    pub label: String,
    pub badge: Option<String>,
    /// Icon name (see `nav_icon_markup`), e.g. `DashboardOutlined`
    pub icon: Option<String>,
    /// Extra route prefixes that also belong to this entry (e.g. its detail forms, or another
    /// route that is served by the same program)
    pub also_active: Vec<String>,
}

impl NavItem {
    pub fn new(href: impl Into<String>, label: impl Into<String>) -> Self {
        Self { href: href.into(), label: label.into(), badge: None, icon: None, also_active: Vec::new() }
    }

    /// Sets the icon (a `db_program.icon` name)
    pub fn with_icon(mut self, icon: Option<String>) -> Self {
        self.icon = icon;
        self
    }

    /// `true` when `pathname` is this entry's route or one of its extra prefixes
    pub fn is_active_for(&self, pathname: &str) -> bool {
        is_nav_active(&self.href, pathname)
            || self.also_active.iter().any(|prefix| is_nav_active(prefix, pathname))
    }

    /// Length of the longest route prefix of this entry that matches `pathname` (0 = none)
    fn match_len(&self, pathname: &str) -> usize {
        std::iter::once(&self.href)
            .chain(self.also_active.iter())
            .filter(|prefix| is_nav_active(prefix, pathname))
            .map(String::len)
            .max()
            .unwrap_or(0)
    }
}

/// A titled group of sidebar links
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavSection {
    pub label: String,
    pub items: Vec<NavItem>,
    /// Icon name of the group
    pub icon: Option<String>,
}

impl NavSection {
    /// Sets the icon (a `db_program.icon` name)
    pub fn with_icon(mut self, icon: Option<String>) -> Self {
        self.icon = icon;
        self
    }

    pub fn new(label: impl Into<String>, items: Vec<NavItem>) -> Self {
        Self { label: label.into(), items, icon: None }
    }
}

/// Returns `true` when `pathname` belongs to the route `href` (exact for `/`, prefix otherwise).
pub fn is_nav_active(href: &str, pathname: &str) -> bool {
    if href == "/" {
        pathname == "/"
    } else {
        pathname == href || pathname.starts_with(&format!("{href}/"))
    }
}

/// Resolves the navigation label for the current pathname (used as breadcrumb title). The most
/// specific (longest) matching `href` wins, so nested routes resolve to their own entry.
pub fn nav_label_for(sections: &[NavSection], pathname: &str) -> Option<String> {
    sections
        .iter()
        .flat_map(|section| section.items.iter())
        .filter(|item| item.is_active_for(pathname))
        .max_by_key(|item| item.match_len(pathname))
        .map(|item| item.label.clone())
}
