use leptos::prelude::*;
use web_package::core::auth::SettingsPage;
use web_package::core::i18n::t_or;

#[component]
pub fn MasterSettingsPage() -> impl IntoView {
    view! { <SettingsPage title=t_or("settings.form.password", "ตั้งรหัสผ่านใหม่") default_redirect="/" /> }
}
