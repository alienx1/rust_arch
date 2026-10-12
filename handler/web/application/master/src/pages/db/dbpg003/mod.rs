//! DBPG003 Page Module (Employees: paged list + create/edit form, Clean UI Pattern)

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::{Dbpg003EmployeeScreen as Dbpg003EmployeePage, Dbpg003Screen as Dbpg003Page};
