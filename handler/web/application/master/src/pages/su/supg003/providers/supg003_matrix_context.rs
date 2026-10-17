//! SUPG003 matrix context: loads a group's matrix, edits it locally (with the rules that WRITE /
//! APPROVE imply READ and that removing READ removes the rest) and saves the whole grant set.

use web_package::core::i18n::t_or;
use leptos::prelude::*;
use crate::audit::track_action;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::pages::su::supg003::models::{
    GrantInput, MatrixRowDto, PermissionGroupDto, SaveMatrixInput,
};
use crate::pages::su::supg003::services::Supg003Service;
use crate::routes::SUPG003;

/// Matrix column a checkbox belongs to
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixColumn {
    Read,
    Write,
    Approve,
}

/// Sets one cell and keeps the row consistent: WRITE / APPROVE need READ, and clearing READ
/// clears them.
pub fn apply_cell(row: &mut MatrixRowDto, column: MatrixColumn, on: bool) {
    match column {
        MatrixColumn::Read => {
            row.read = on;
            if !on {
                row.write = false;
                row.approve = false;
            }
        }
        MatrixColumn::Write => {
            row.write = on;
            if on {
                row.read = true;
            }
        }
        MatrixColumn::Approve => {
            row.approve = on;
            if on {
                row.read = true;
            }
        }
    }
}

/// State of the matrix editor (`/su/supg003/group/:id/matrix`)
#[derive(Clone, Copy)]
pub struct Supg003MatrixContext {
    pub group: ReadSignal<Option<PermissionGroupDto>>,
    pub rows: ReadSignal<Vec<MatrixRowDto>>,
    /// Edited since the last load / save
    pub is_dirty: ReadSignal<bool>,
    pub is_loading: ReadSignal<bool>,
    pub is_saving: ReadSignal<bool>,
    pub error: ReadSignal<Option<String>>,
    pub notice: ReadSignal<Option<String>>,
    pub set_cell: Callback<(String, MatrixColumn, bool)>,
    /// Grant or clear every action of every program of one module (`system_code`, `module_code`)
    pub set_module: Callback<(String, String, bool)>,
    /// Grant or clear one column for every program
    pub set_column: Callback<(MatrixColumn, bool)>,
    pub reload: Callback<()>,
    pub save: Callback<()>,
    pub back: Callback<()>,
}

/// Provides the matrix context for group `id`
pub fn provide_supg003_matrix_context(id: String) -> Supg003MatrixContext {
    let service = Supg003Service::new();
    let (group, set_group) = signal(Option::<PermissionGroupDto>::None);
    let (rows, set_rows) = signal(Vec::<MatrixRowDto>::new());
    let (is_dirty, set_dirty) = signal(false);
    let (is_loading, set_is_loading) = signal(true);
    let (is_saving, set_is_saving) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (notice, set_notice) = signal(Option::<String>::None);
    let (reload_trigger, set_reload_trigger) = signal(0u32);

    let navigate = use_navigate();
    let back = Callback::new(move |()| navigate(SUPG003, Default::default()));

    Effect::new({
        let service = service.clone();
        let id = id.clone();
        move |_| {
            let _ = reload_trigger.get();
            set_is_loading.set(true);
            set_error.set(None);
            let s = service.clone();
            let id = id.clone();
            spawn_local(async move {
                match s.fetch_matrix(&id).await {
                    Ok(m) => {
                        track_action("SUPG003.matrix.save", &id);
                        set_group.set(Some(m.group));
                        set_rows.set(m.rows);
                        set_dirty.set(false);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
                set_is_loading.set(false);
            });
        }
    });

    let edit = move |f: &dyn Fn(&mut Vec<MatrixRowDto>)| {
        set_rows.update(|r| f(r));
        set_dirty.set(true);
        set_notice.set(None);
    };

    let set_cell = Callback::new(move |(program_id, column, on): (String, MatrixColumn, bool)| {
        edit(&|rows| {
            if let Some(row) = rows.iter_mut().find(|r| r.program_id == program_id) {
                apply_cell(row, column, on);
            }
        });
    });
    let set_module = Callback::new(move |(system, module, on): (String, String, bool)| {
        edit(&|rows| {
            for row in rows.iter_mut().filter(|r| r.system_code == system && r.module_code == module) {
                for column in [MatrixColumn::Read, MatrixColumn::Write, MatrixColumn::Approve] {
                    apply_cell(row, column, on);
                }
            }
        });
    });
    let set_column = Callback::new(move |(column, on): (MatrixColumn, bool)| {
        edit(&|rows| {
            for row in rows.iter_mut() {
                apply_cell(row, column, on);
            }
        });
    });

    let reload = Callback::new(move |()| set_reload_trigger.update(|v| *v = v.wrapping_add(1)));

    let save = Callback::new({
        let id = id.clone();
        move |()| {
            let s = service.clone();
            let id = id.clone();
            let grants = rows
                .get_untracked()
                .into_iter()
                .map(|r| GrantInput { program_id: r.program_id, read: r.read, write: r.write, approve: r.approve })
                .collect::<Vec<_>>();
            let Some(row_version) = group.get_untracked().map(|g| g.row_version) else {
                set_error.set(Some(t_or("supg003.matrix.err_not_loaded", "ยังโหลดข้อมูลไม่สำเร็จ ไม่สามารถบันทึกได้")));
                return;
            };
            set_is_saving.set(true);
            set_error.set(None);
            set_notice.set(None);
            spawn_local(async move {
                match s.save_matrix(&id, SaveMatrixInput { grants, row_version }).await {
                    Ok(m) => {
                        set_group.set(Some(m.group));
                        set_rows.set(m.rows);
                        set_dirty.set(false);
                        set_notice.set(Some(t_or("supg003.matrix.saved", "บันทึกสิทธิ์เรียบร้อย")));
                    }
                    Err(e) => set_error.set(Some(e)),
                }
                set_is_saving.set(false);
            });
        }
    });

    let ctx = Supg003MatrixContext {
        group,
        rows,
        is_dirty,
        is_loading,
        is_saving,
        error,
        notice,
        set_cell,
        set_module,
        set_column,
        reload,
        save,
        back,
    };
    provide_context(ctx);
    ctx
}

pub fn use_supg003_matrix() -> Supg003MatrixContext {
    use_context::<Supg003MatrixContext>()
        .expect("Supg003MatrixContext must be provided by provide_supg003_matrix_context()")
}
