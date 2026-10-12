//! Master Application Login Page

use leptos::prelude::*;
use web_package::core::i18n::t_or;
use web_package::core::auth::LoginPage;

#[component]
pub fn MasterLoginPage() -> impl IntoView {
    view! {
        <LoginPage
            title=t_or("login.title", "เข้าสู่ระบบ Master Data")
            subtitle=t_or("login.subtitle", "ระบบจัดการข้อมูลหลักและการยืนยันตัวตน (Master Data & IAM Engine)")
            default_redirect="/"
        />
    }
}
