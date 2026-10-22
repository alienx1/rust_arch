//! Which requests are "a user changed data" and what to tell them. Pure functions of method, path
//! and status (unit-tested in `tests/change_notice_test.rs`).

use crate::middleware::audit::audit_target::{api_segments, is_program_segment};

/// What happened to the data
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    Created,
    Updated,
    Deleted,
    /// The request was refused or failed
    Failed,
}

/// What the user is told about one write request
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeNotice {
    /// Program code, e.g. `DBPG003`
    pub program: String,
    /// The resource the request wrote, e.g. `employees` (first path segment after the program)
    pub target: String,
    pub kind: ChangeKind,
    /// HTTP status of the answer
    pub status: u16,
    /// Web path of the program, e.g. `/db/dbpg003`
    pub link: String,
}

/// Programs whose writes are not data entry: the audit page (exports, client events), the report
/// list and the notification list, which tell the user themselves
const QUIET_PROGRAMS: [&str; 3] = ["ALPG000", "RPPG000", "NTPG000"];

/// Sub-resources saved together with their parent (the form saves the record, then these): the
/// parent's notice covers them
const PART_OF_PARENT: [&str; 3] = ["contacts", "image", "images"];

/// The notice for a finished request, `None` when nobody needs telling: reads, other than
/// program APIs, programs that report on their own, and the sub-resources of a form save.
/// A refusal that is not about the data (401: no session) is not a change either.
pub fn describe_change(method: &str, path: &str, status: u16) -> Option<ChangeNotice> {
    let method = method.to_ascii_uppercase();
    if !matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") || status == 401 {
        return None;
    }
    let segs = api_segments(path)?;
    let at = segs.iter().position(|s| is_program_segment(s))?;
    let program = segs[at].to_ascii_uppercase();
    if QUIET_PROGRAMS.contains(&program.as_str()) {
        return None;
    }
    let below: Vec<&str> = segs[at + 1..].to_vec();
    let target = below.first().copied().unwrap_or("").to_string();
    if below.last().is_some_and(|l| PART_OF_PARENT.contains(l)) && below.len() > 1 {
        return None;
    }
    let kind = if status >= 400 {
        ChangeKind::Failed
    } else {
        match method.as_str() {
            "DELETE" => ChangeKind::Deleted,
            // POST on a collection creates; POST on something below it is an action on a record
            "POST" if below.len() <= 1 => ChangeKind::Created,
            _ => ChangeKind::Updated,
        }
    };
    let link = format!("/{}", segs[..=at].iter().map(|s| s.to_ascii_lowercase()).collect::<Vec<_>>().join("/"));
    Some(ChangeNotice { program, target, kind, status, link })
}
