//! SUPG003 Page Module (Permission Matrix: group list, group form, matrix editor, Clean UI Pattern)

pub mod components;
pub mod models;
pub mod providers;
pub mod screens;
pub mod services;

pub use screens::{
    Supg003GroupScreen as Supg003GroupPage, Supg003MatrixScreen as Supg003MatrixPage,
    Supg003Screen as Supg003Page,
};
