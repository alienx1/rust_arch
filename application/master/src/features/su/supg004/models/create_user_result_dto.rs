//! SUPG004 create-account result DTO

use serde::{Deserialize, Serialize};

use super::recovery_link_dto::RecoveryLinkDto;
use super::user_account_dto::UserAccountDto;

/// Outcome of creating an account. The account exists even when no recovery link could be made;
/// issue one later from the list.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateUserResultDto {
    pub user: UserAccountDto,
    pub recovery_link: Option<RecoveryLinkDto>,
    pub recovery_error: Option<String>,
}
