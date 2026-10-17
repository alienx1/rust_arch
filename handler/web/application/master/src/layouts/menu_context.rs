use leptos::prelude::*;
use leptos::task::spawn_local;
use web_package::core::auth::use_auth;
use web_package::core::i18n::use_i18n;
use web_package::layout::{NavSection, UserMenuItem};

use crate::access::use_access;
use crate::layouts::master_navigation::{fallback_sections, menu_to_sections, user_menu_items};
use crate::layouts::menu_service::{MenuNodeDto, MenuService};
use crate::pages::demo::is_dev_env;

/// Reactive sidebar menu
#[derive(Clone, Copy)]
pub struct MenuContext {
    /// Sections for `MainLayout` (empty until the first load finishes)
    pub sections: ReadSignal<Vec<NavSection>>,
    /// Entries of the topbar's user dropdown (my reports, notifications)
    pub user_menu: ReadSignal<Vec<UserMenuItem>>,
}

/// Provides the menu context; reloads when the signed-in state or language changes.
pub fn provide_menu_context() -> MenuContext {
    let auth = use_auth();
    let i18n = use_i18n();
    let access = use_access();
    let service = MenuService::new();
    let (sections, set_sections) = signal(Vec::<NavSection>::new());
    let (user_menu, set_user_menu) = signal(Vec::<UserMenuItem>::new());
    // The tree the sections were built from, to rebuild only when it (or the language) changed
    let last_built = StoredValue::new(Option::<(Vec<MenuNodeDto>, String)>::None);

    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            return;
        }
        let lang = i18n.current_lang.get();
        let _ = i18n.dict.get(); // track dictionary updates
        let _ = access.version.get(); // background re-check: a rights change reshapes the menu
        let service = service.clone();
        spawn_local(async move {
            let show_dev_demo = is_dev_env();
            let result = service.fetch_menu_tree().await;
            // The layout can be gone by the time the answer arrives (sign-out, navigation to
            // /login, a hot reload): its signals are disposed, so write with `try_*` and drop the
            // answer instead of panicking
            match result {
                Ok(tree) => {
                    let built = (tree, lang.clone());
                    let Some(previous) = last_built.try_get_value() else { return };
                    if previous.as_ref() != Some(&built) {
                        let _ = set_sections.try_set(menu_to_sections(&built.0, &lang, show_dev_demo));
                        let _ = set_user_menu.try_set(user_menu_items(&built.0, &lang));
                        last_built.try_set_value(Some(built));
                    }
                }
                Err(e) => {
                    leptos::logging::warn!("menu: {e}; showing the fallback menu");
                    let _ = set_sections.try_set(fallback_sections(&lang, show_dev_demo));
                }
            }
        });
    });

    let ctx = MenuContext { sections, user_menu };
    provide_context(ctx);
    ctx
}
