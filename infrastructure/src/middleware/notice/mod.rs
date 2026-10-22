//! Change notices: tell the user how their create / update / delete went (see [`change_notice`]).

pub mod change_notice;
pub mod change_target;

pub use change_notice::{ChangeNoticeState, change_notice};
pub use change_target::{ChangeKind, ChangeNotice, describe_change};
