//! DBPG002 Providers Module

pub mod dbpg002_context;
pub mod dbpg002_form_context;

pub use dbpg002_context::{Dbpg002Tab, provide_dbpg002_context, use_dbpg002};
pub use dbpg002_form_context::{
    provide_dbpg002_branch_form_context, provide_dbpg002_department_form_context,
    use_dbpg002_branch_form, use_dbpg002_department_form,
};
