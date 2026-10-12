//! DBPG002 Page Module (Branches & Departments: list + create/edit forms, Clean UI Pattern)

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::{
    Dbpg002BranchScreen as Dbpg002BranchPage, Dbpg002DepartmentScreen as Dbpg002DepartmentPage,
    Dbpg002Screen as Dbpg002Page,
};
