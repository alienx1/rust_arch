//! SUPG004 create-account result DTO

use serde::{Deserialize, Serialize};

use super::user_account_dto::UserAccountDto;
use crate::features::su::account_security::PasswordMailResult;

/// Outcome of creating an account. The account exists even when the mail could not be sent: the
/// result then carries the link to hand over, and a mail can be sent again from the list.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateUserResultDto {
    pub user: UserAccountDto,
    /// The e-mail with the link to set the first password
    pub password_mail: PasswordMailResult,
}
