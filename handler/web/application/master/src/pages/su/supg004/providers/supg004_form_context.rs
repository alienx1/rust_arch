//! SUPG004 account form context: loads the account to edit and the group options, searches
//! employees without an account, saves, and returns to the list. A newly created account stays
//! on the page to show the outcome of its set-password mail.

use web_package::core::i18n::t_or;
use leptos::prelude::*;

use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg004::models::{
    CreateUserInput, CreateUserResultDto, EmployeeOptionDto, GroupOptionDto, PasswordMailResultDto, ScopeOptionsDto, UpdateUserInput,
    UserAccountDto, UserFormInput,
};
use crate::pages::su::supg004::services::Supg004Service;
use crate::routes::SUPG004;

/// State of the account form (`/su/supg004/user[/:id]`)
#[derive(Clone, Copy)]
pub struct Supg004FormContext {
    pub editing: ReadSignal<Option<UserAccountDto>>,
    pub groups: ReadSignal<Vec<GroupOptionDto>>,
    pub scope_options: ReadSignal<ScopeOptionsDto>,
    pub employees: ReadSignal<Vec<EmployeeOptionDto>>,
    pub created: ReadSignal<Option<CreateUserResultDto>>,
    /// Outcome of the last password mail asked for while editing
    pub mail_outcome: ReadSignal<Option<PasswordMailResultDto>>,
    /// The account cannot sign in until a new password is set
    pub must_change: ReadSignal<bool>,
    pub is_mailing: ReadSignal<bool>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub is_deleting: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub search_employees: Callback<String>,
    pub save: Callback<UserFormInput>,
    pub cancel: Callback<()>,
    pub delete: Callback<()>,
    /// Mail a set-password link (reset); the old password keeps working
    pub send_password_mail: Callback<()>,
    /// Block sign-in until a new password is set, sign the user out everywhere and mail the link
    pub force_password_change: Callback<()>,
    pub dismiss_mail: Callback<()>,
}

/// Provides the account form context; `id` is `Some` when editing.
pub fn provide_supg004_form_context(id: Option<String>) -> Supg004FormContext {
    let service = Supg004Service::new();
    let (editing, set_editing) = signal(Option::<UserAccountDto>::None);
    let (groups, set_groups) = signal(Vec::<GroupOptionDto>::new());
    let (scope_options, set_scope_options) = signal(ScopeOptionsDto::default());
    let (employees, set_employees) = signal(Vec::<EmployeeOptionDto>::new());
    let (created, set_created) = signal(Option::<CreateUserResultDto>::None);
    let (mail_outcome, set_mail_outcome) = signal(Option::<PasswordMailResultDto>::None);
    let (must_change, set_must_change) = signal(false);
    let (is_mailing, set_is_mailing) = signal(false);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (is_deleting, set_is_deleting) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    // Latest employee search wins; slower older answers are dropped
    let (search_seq, set_search_seq) = signal(0u64);

    let navigate = use_navigate();
    let cancel = Callback::new(move |()| navigate(SUPG004, Default::default()));

    {
        let s = service.clone();
        let id = id.clone();
        spawn_local(async move {
            let mut err: Option<String> = None;
            match s.fetch_groups().await {
                Ok(v) => set_groups.set(v),
                Err(e) => err = Some(e),
            }
            match s.fetch_scope_options().await {
                Ok(v) => set_scope_options.set(v),
                Err(e) => err = err.or(Some(e)),
            }
            match id {
                Some(id) => match s.fetch_user(&id).await {
                    Ok(v) => {
                        set_must_change.set(v.must_change_password);
                        set_editing.set(Some(v))
                    }
                    Err(e) => err = err.or(Some(e)),
                },
                None => match s.fetch_employees("").await {
                    Ok(v) => set_employees.set(v),
                    Err(e) => err = err.or(Some(e)),
                },
            }
            if let Some(e) = err {
                set_error.set(Some(e));
            }
            set_is_loading.set(false);
        });
    }

    let search_employees = Callback::new({
        let service = service.clone();
        move |term: String| {
            let seq = search_seq.get_untracked() + 1;
            set_search_seq.set(seq);
            let s = service.clone();
            spawn_local(async move {
                if let Ok(v) = s.fetch_employees(&term).await {
                    if search_seq.get_untracked() == seq {
                        set_employees.set(v);
                    }
                }
            });
        }
    });

    let delete = Callback::new({
        let service = service.clone();
        let id = id.clone();
        move |()| {
            let Some(id) = id.clone() else { return };
            let s = service.clone();
            set_is_deleting.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = s.delete_user(&id).await;
                set_is_deleting.set(false);
                match res {
                    Ok(()) => {
                        track_action("SUPG004.delete", "");
                        cancel.run(())
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });

    let send_password_mail = Callback::new({
        let service = service.clone();
        let id = id.clone();
        move |()| {
            let Some(id) = id.clone() else { return };
            let s = service.clone();
            set_is_mailing.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = s.send_password_mail(&id).await;
                set_is_mailing.set(false);
                match res {
                    Ok(r) => {
                        track_action("SUPG004.password_mail", &id);
                        set_mail_outcome.set(Some(r));
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });
    let force_password_change = Callback::new({
        let service = service.clone();
        let id = id.clone();
        move |()| {
            let Some(id) = id.clone() else { return };
            let s = service.clone();
            set_is_mailing.set(true);
            set_error.set(None);
            spawn_local(async move {
                let res = s.force_password_change(&id).await;
                set_is_mailing.set(false);
                match res {
                    Ok(r) => {
                        track_action("SUPG004.force_password_change", &id);
                        set_must_change.set(true);
                        set_mail_outcome.set(Some(r));
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    });
    let dismiss_mail = Callback::new(move |()| set_mail_outcome.set(None));

    let save = Callback::new(move |input: UserFormInput| {
        let s = service.clone();
        let id = id.clone();
        let current = editing.get_untracked();
        set_is_saving.set(true);
        set_error.set(None);
        spawn_local(async move {
            let res: Result<bool, String> = match (id, current) {
                (Some(id), Some(user)) => s
                    .update_user(
                        &id,
                        UpdateUserInput {
                            assignments: input.assignments,
                            active: input.active,
                            row_version: user.row_version,
                        },
                    )
                    .await
                    .map(|_| true),
                (Some(_), None) => Err(t_or("supg004.form.err_not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")),
                (None, _) => s
                    .create_user(CreateUserInput {
                        employee_id: input.employee_id,
                        username: input.username,
                        email: input.email,
                        assignments: input.assignments,
                        active: input.active,
                    })
                    .await
                    .map(|result| {
                        set_created.set(Some(result));
                        false
                    }),
            };
            set_is_saving.set(false);
            match res {
                Ok(true) => {
                    track_action("SUPG004.save", "");
                    cancel.run(())
                }
                Ok(false) => {}
                Err(e) => set_error.set(Some(e)),
            }
        });
    });

    let ctx = Supg004FormContext {
        editing,
        groups,
        scope_options,
        employees,
        created,
        mail_outcome,
        must_change,
        is_mailing,
        is_loading,
        is_saving,
        is_deleting,
        error,
        search_employees,
        save,
        cancel,
        delete,
        send_password_mail,
        force_password_change,
        dismiss_mail,
    };
    provide_context(ctx);
    ctx
}

pub fn use_supg004_form() -> Supg004FormContext {
    use_context::<Supg004FormContext>()
        .expect("Supg004FormContext must be provided by provide_supg004_form_context()")
}
