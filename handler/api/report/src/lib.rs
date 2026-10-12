//! Report API Service (api-report): the report worker (consumes `report.requests` from Redpanda,
//! writes the files to object storage) and the endpoints the web app reads them through.

pub mod connection;
pub mod routers;

pub use connection::ReportConnection;
pub use routers::build_app_router;
