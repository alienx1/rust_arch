//! Axum Routers for api-master.

pub mod access;
pub mod access_router;
pub mod alpg000_router;
pub mod alpg001_router;
pub mod audit_events_router;
pub mod auth_gateway_router;
pub mod base_router;
pub mod contacts_router;
pub mod dashboard_router;
pub mod dbpg000_router;
pub mod dbpg001_router;
pub mod dbpg002_router;
pub mod dbpg003_router;
pub mod dbpg004_router;
pub mod dbpg005_router;
pub mod dbpg006_router;
pub mod kratos_hook_router;
pub mod localization_router;
pub mod supg001_router;
pub mod supg002_router;
pub mod supg003_router;
pub mod supg004_router;
pub mod supg005_router;

pub use alpg001_router::{
    ALPG001_PATH, Alpg001ApiError, Alpg001ErrorResponse, Alpg001State, alpg001_routes,
    alpg001_routes_with_state,
};
pub use alpg000_router::{
    ALPG000_PATH, Alpg000ApiError, Alpg000ErrorResponse, Alpg000State, alpg000_routes,
    alpg000_routes_with_state,
};
pub use access_router::{ACCESS_PATH, AccessState, access_routes, access_routes_with_state};
pub use audit_events_router::{
    AUDIT_EVENTS_PATH, AuditEventsState, audit_events_routes, audit_events_routes_with_state,
    client_actor_of,
};
pub use auth_gateway_router::{
    AUTH_GATEWAY_PATH, GatewayState, GatewayTarget, auth_gateway_routes, auth_gateway_routes_with_state,
};
pub use base_router::*;
pub use dashboard_router::*;
pub use dbpg000_router::{
    DBPG000_PATH, Dbpg000ApiError, Dbpg000ErrorResponse, Dbpg000State, dbpg000_routes,
    dbpg000_routes_with_state,
};
pub use dbpg001_router::{
    DBPG001_PATH, Dbpg001ApiError, Dbpg001ErrorResponse, Dbpg001State, dbpg001_routes,
    dbpg001_routes_with_state,
};
pub use dbpg002_router::{
    DBPG002_PATH, Dbpg002ApiError, Dbpg002ErrorResponse, Dbpg002State, dbpg002_routes,
    dbpg002_routes_with_state,
};
pub use dbpg003_router::{
    DBPG003_PATH, Dbpg003ApiError, Dbpg003ErrorResponse, Dbpg003State, dbpg003_routes,
    dbpg003_routes_with_state,
};
pub use dbpg004_router::{
    DBPG004_PATH, Dbpg004ApiError, Dbpg004ErrorResponse, Dbpg004State, dbpg004_routes,
    dbpg004_routes_with_state,
};
pub use dbpg005_router::{
    DBPG005_PATH, Dbpg005ApiError, Dbpg005ErrorResponse, Dbpg005State, dbpg005_routes,
    dbpg005_routes_with_state,
};
pub use dbpg006_router::{
    DBPG006_PATH, Dbpg006ApiError, Dbpg006ErrorResponse, Dbpg006State, dbpg006_routes,
    dbpg006_routes_with_state,
};
pub use kratos_hook_router::*;
pub use localization_router::*;
pub use supg001_router::{
    SUPG001_PATH, Supg001ApiError, Supg001ErrorResponse, Supg001State, supg001_routes,
    supg001_routes_with_state,
};
pub use supg002_router::{
    SUPG002_PATH, Supg002ApiError, Supg002ErrorResponse, Supg002State, get_menu_handler,
    supg002_routes, supg002_routes_with_state, username_of,
};
pub use supg003_router::{
    SUPG003_PATH, Supg003ApiError, Supg003ErrorResponse, Supg003State, supg003_routes,
    supg003_routes_with_state,
};
pub use supg004_router::{
    SUPG004_PATH, Supg004ApiError, Supg004ErrorResponse, Supg004State, supg004_routes,
    supg004_routes_with_state,
};
pub use supg005_router::*;
