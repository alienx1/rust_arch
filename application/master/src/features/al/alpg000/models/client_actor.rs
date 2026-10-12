//! ALPG000 who reported client events (filled by the server from the session)

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClientActor {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub session_id: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub language: Option<String>,
}
