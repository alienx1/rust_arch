//! SUPG006 Screen (`/su/supg006`): security & mail settings
//!
//! Two tabs (sign-in lock, mail templates) over one set of edits; saving sends only what changed,
//! after a second confirmation.

use leptos::prelude::*;
use web_package::core::i18n::{t_fmt, t_or};
use web_package::layout::{PageActions, PageContainer, PageContent, PageDescription, PageHeader, PageTitle};
use web_package::ui::component::{Button, ButtonSize, ButtonVariant, SaveConfirmButton, Spinner, StatusColor, StatusDot};

use crate::access::{Action, PermissionGate};
use crate::pages::su::supg006::components::{LoginLockCard, MailTemplateEditor, PasswordPolicyCard};
use crate::pages::su::supg006::providers::provide_supg006_context;

#[component]
pub fn Supg006Screen() -> impl IntoView {
    let ctx = provide_supg006_context();
    let (tab, set_tab) = signal("login".to_string());

    let tab_button = move |code: &'static str, label: Signal<String>| {
        view! {
            <button
                type="button"
                role="tab"
                class=move || if tab.get() == code {
                    "px-4 py-2 text-sm font-medium border-b-2 border-primary text-foreground cursor-pointer"
                } else {
                    "px-4 py-2 text-sm font-medium border-b-2 border-transparent text-muted-foreground hover:text-foreground cursor-pointer"
                }
                on:click=move |_| set_tab.set(code.to_string())
            >
                {move || label.get()}
            </button>
        }
    };

    view! {
        <PageContainer>
            <PageHeader>
                <div>
                    <PageTitle>{move || t_or("supg006.title", "ตั้งค่าความปลอดภัยและอีเมล (Security & Mail Settings)")}</PageTitle>
                    <PageDescription>
                        {move || t_or("supg006.description", "กำหนดการล็อกบัญชีเมื่อใส่รหัสผ่านผิดหลายครั้ง นโยบายรหัสผ่าน และเทมเพลตอีเมลตั้งรหัสผ่าน • รหัสโปรแกรม: SUPG006")}
                    </PageDescription>
                </div>
                <PageActions>
                    {move || if ctx.error.get().is_some() {
                        view! { <StatusDot color=StatusColor::Destructive label="Master DB: Disconnected" /> }.into_any()
                    } else {
                        view! { <StatusDot color=StatusColor::Success label="Master DB: Connected" /> }.into_any()
                    }}
                </PageActions>
            </PageHeader>

            <PageContent>
                {move || ctx.error.get().map(|err| view! {
                    <div class="p-3.5 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-xs shadow-sm mb-4">
                        <span class="font-medium">{err}</span>
                    </div>
                })}
                {move || ctx.notice.get().map(|msg| view! {
                    <div class="p-3 text-xs rounded-md bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30 mb-4">{msg}</div>
                })}
                {move || ctx.is_loading.get().then(|| view! {
                    <div class="p-8 flex items-center justify-center gap-3 text-muted-foreground">
                        <Spinner class="h-6 w-6 text-primary" />
                        <span class="text-sm">{t_or("common.state.loading", "กำลังโหลดข้อมูล...")}</span>
                    </div>
                })}

                <div class="flex border-b border-border mb-4" role="tablist">
                    {tab_button("login", Signal::derive(move || t_or("supg006.tab.login", "การป้องกันการเดารหัสผ่าน")))}
                    {tab_button("policy", Signal::derive(move || t_or("supg006.tab.policy", "นโยบายรหัสผ่าน")))}
                    {tab_button("mail", Signal::derive(move || t_or("supg006.tab.mail", "เทมเพลตอีเมล")))}
                </div>

                {move || match tab.get().as_str() {
                    "login" => view! { <LoginLockCard /> }.into_any(),
                    "policy" => view! { <PasswordPolicyCard /> }.into_any(),
                    _ => view! { <MailTemplateEditor /> }.into_any(),
                }}

                <PermissionGate program="SUPG006" action=Action::Write>
                    <div class="mt-5 flex flex-wrap items-center gap-3 justify-end">
                        <span class="text-xs text-muted-foreground">
                            {move || {
                                let n = ctx.changed_count();
                                if n == 0 {
                                    t_or("supg006.no_changes", "ยังไม่มีการเปลี่ยนแปลง")
                                } else {
                                    t_fmt("supg006.changed_count", "แก้ไขแล้ว {n} รายการ (ยังไม่ได้บันทึก)", &[("n", &n.to_string())])
                                }
                            }}
                        </span>
                        <Button
                            variant=ButtonVariant::Outline
                            size=ButtonSize::Sm
                            disabled=Signal::derive(move || ctx.changed_count() == 0 || ctx.is_saving.get())
                            on_click=Callback::new(move |_| ctx.discard.run(()))
                        >
                            {t_or("supg006.discard", "ยกเลิกการแก้ไข")}
                        </Button>
                        <SaveConfirmButton
                            label=Signal::derive(move || if ctx.is_saving.get() { t_or("supg006.saving", "กำลังบันทึก...") } else { t_or("common.action.save", "บันทึกข้อมูล") })
                            on_confirm=Callback::new(move |_| ctx.save.run(()))
                            busy=Signal::derive(move || ctx.is_saving.get())
                            disabled=Signal::derive(move || ctx.changed_count() == 0)
                        />
                    </div>
                </PermissionGate>
            </PageContent>
        </PageContainer>
    }
}
