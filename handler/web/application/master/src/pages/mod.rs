//! Application Pages

pub mod al;
pub mod dashboard;
pub mod db;
pub mod demo;
pub mod login;
pub mod settings;
pub mod not_found;
pub mod nt;
pub mod rp;
pub mod su;

pub use al::alpg000::{Alpg000LogPage, Alpg000Page};
pub use al::alpg001::Alpg001Page;
pub use dashboard::DashboardPage;
pub use db::dbpg000::Dbpg000Page;
pub use db::dbpg001::{Dbpg001CompanyPage, Dbpg001OrgPage};
pub use db::dbpg002::{Dbpg002BranchPage, Dbpg002DepartmentPage, Dbpg002Page};
pub use db::dbpg003::{Dbpg003EmployeePage, Dbpg003Page};
pub use db::dbpg004::{Dbpg004LookupPage, Dbpg004Page};
pub use db::dbpg005::{Dbpg005FormPage, Dbpg005Page};
pub use db::dbpg006::{Dbpg006LabelPage, Dbpg006Page};
pub use demo::DemoPage;
pub use login::MasterLoginPage;
pub use settings::MasterSettingsPage;
pub use not_found::NotFoundPage;
pub use nt::ntpg000::Ntpg000Page;
pub use rp::rppg000::Rppg000Page;
pub use su::supg001::{Supg001FormPage, Supg001Page};
pub use su::supg002::{Supg002Page, Supg002ProgramPage};
pub use su::supg003::{Supg003GroupPage, Supg003MatrixPage, Supg003Page};
pub use su::supg004::{Supg004Page, Supg004UserPage};
pub use su::supg005::Supg005Page;
