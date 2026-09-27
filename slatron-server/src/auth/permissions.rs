use crate::db::DbConnection;
use crate::models::{Permission, User};
use axum::http::StatusCode;
use diesel::prelude::*;
use std::collections::HashSet;

/// Resource types enforced by object-level permissions.
/// These values are stored in `permissions.resource_type`.
pub const RES_CONTENT: &str = "content";
pub const RES_SCHEDULE: &str = "schedule";
pub const RES_NODE: &str = "node";

pub const RESOURCE_TYPES: &[&str] = &[RES_CONTENT, RES_SCHEDULE, RES_NODE];

/// `resource_id` value meaning "every object of this type".
pub const WILDCARD_ID: i32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Read,
    Write,
    Admin,
}

impl Level {
    fn parse(s: &str) -> Option<Level> {
        match s {
            "read" => Some(Level::Read),
            "write" => Some(Level::Write),
            "admin" => Some(Level::Admin),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
}

/// Pure decision core.
///
/// - `admin` role: unrestricted.
/// - `editor` role: unrestricted on enforced resource types (preserves the
///   pre-permissions behavior where editors had full access).
/// - any other role (viewer): grant-driven, deny-by-default. Read requires a
///   grant of level read or higher; write requires write or higher.
pub fn decide(role: &str, grant: Option<Level>, action: Action) -> bool {
    match role {
        "admin" | "editor" => true,
        _ => match action {
            Action::Read => grant.is_some_and(|g| g >= Level::Read),
            Action::Write => grant.is_some_and(|g| g >= Level::Write),
        },
    }
}

/// Loaded permission rows for one user, with the checks handlers need.
pub struct Grants {
    rows: Vec<Permission>,
}

impl Grants {
    /// Empty grant set (used for admins/editors, whose role already allows
    /// everything on enforced types — avoids a DB round trip per request).
    pub fn none() -> Grants {
        Grants { rows: Vec::new() }
    }

    pub fn load(conn: &mut DbConnection, uid: i32) -> QueryResult<Grants> {
        use crate::schema::permissions::dsl::*;
        let rows = permissions
            .filter(user_id.eq(uid))
            .select(Permission::as_select())
            .load(conn)?;
        Ok(Grants { rows })
    }

    /// Load grants appropriate for `user`: editors/admins get an empty set.
    pub fn for_user(conn: &mut DbConnection, user: &User) -> QueryResult<Grants> {
        if user.is_editor() {
            Ok(Grants::none())
        } else {
            Grants::load(conn, user.id.unwrap_or(0))
        }
    }

    /// Highest grant level covering (resource_type, resource_id), including
    /// the type-wide wildcard.
    fn best(&self, res_type: &str, res_id: i32) -> Option<Level> {
        self.rows
            .iter()
            .filter(|r| r.resource_type == res_type && (r.resource_id == res_id || r.resource_id == WILDCARD_ID))
            .filter_map(|r| Level::parse(&r.permission_level))
            .max()
    }

    pub fn can(&self, role: &str, action: Action, res_type: &str, res_id: i32) -> bool {
        decide(role, self.best(res_type, res_id), action)
    }

    /// 403 unless the role+grants allow `action` on the object.
    pub fn ensure(&self, role: &str, action: Action, res_type: &str, res_id: i32) -> Result<(), StatusCode> {
        if self.can(role, action, res_type, res_id) {
            Ok(())
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }

    /// Object ids of `res_type` the role may read. `None` means unrestricted
    /// (admins/editors, or a wildcard read grant). Viewers with only specific
    /// grants get the exact id set — possibly empty (deny-by-default).
    pub fn visible_ids(&self, role: &str, res_type: &str) -> Option<HashSet<i32>> {
        if role == "admin" || role == "editor" {
            return None;
        }
        let mut ids = HashSet::new();
        for row in &self.rows {
            if row.resource_type != res_type {
                continue;
            }
            if Level::parse(&row.permission_level).is_some_and(|l| l >= Level::Read) {
                if row.resource_id == WILDCARD_ID {
                    return None;
                }
                ids.insert(row.resource_id);
            }
        }
        Some(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perm(res_id: i32, level: &str) -> Permission {
        Permission {
            id: None,
            user_id: 1,
            resource_type: RES_CONTENT.to_string(),
            resource_id: res_id,
            permission_level: level.to_string(),
            created_at: chrono::naive::NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        }
    }

    #[test]
    fn admin_and_editor_bypass_grants() {
        let g = Grants::none();
        assert!(decide("admin", None, Action::Write));
        assert!(decide("editor", None, Action::Write));
        assert!(g.can("admin", Action::Write, RES_CONTENT, 7));
        assert!(g.can("editor", Action::Write, RES_SCHEDULE, 7));
    }

    #[test]
    fn viewer_denied_by_default() {
        let g = Grants::none();
        assert!(!g.can("viewer", Action::Read, RES_CONTENT, 7));
        assert!(!g.can("viewer", Action::Write, RES_CONTENT, 7));
    }

    #[test]
    fn viewer_exact_grants() {
        let g = Grants { rows: vec![perm(7, "write")] };
        assert!(g.can("viewer", Action::Read, RES_CONTENT, 7));
        assert!(g.can("viewer", Action::Write, RES_CONTENT, 7));
        assert!(!g.can("viewer", Action::Read, RES_CONTENT, 8));
        // Grants are per-type: a content grant says nothing about nodes.
        assert!(!g.can("viewer", Action::Read, RES_NODE, 7));
    }

    #[test]
    fn viewer_read_grant_does_not_imply_write() {
        let g = Grants { rows: vec![perm(7, "read")] };
        assert!(g.can("viewer", Action::Read, RES_CONTENT, 7));
        assert!(!g.can("viewer", Action::Write, RES_CONTENT, 7));
    }

    #[test]
    fn wildcard_grant_covers_all_objects() {
        let g = Grants { rows: vec![perm(WILDCARD_ID, "read")] };
        assert!(g.can("viewer", Action::Read, RES_CONTENT, 42));
        assert!(!g.can("viewer", Action::Write, RES_CONTENT, 42));
    }

    #[test]
    fn best_takes_maximum_level() {
        let g = Grants {
            rows: vec![perm(7, "read"), perm(WILDCARD_ID, "write")],
        };
        assert!(g.can("viewer", Action::Write, RES_CONTENT, 7));
        assert!(g.can("viewer", Action::Write, RES_CONTENT, 99));
    }

    #[test]
    fn visible_ids_semantics() {
        // Editors unrestricted.
        assert_eq!(Grants::none().visible_ids("editor", RES_CONTENT), None);
        // Wildcard read => unrestricted.
        let w = Grants { rows: vec![perm(WILDCARD_ID, "read")] };
        assert_eq!(w.visible_ids("viewer", RES_CONTENT), None);
        // Exact grants => exact set.
        let e = Grants { rows: vec![perm(3, "read"), perm(4, "write"), perm(5, "read")] };
        let ids = e.visible_ids("viewer", RES_CONTENT).unwrap();
        assert!(ids == HashSet::from([3, 4, 5]));
        // No grants => empty set, not unrestricted.
        assert_eq!(Grants::none().visible_ids("viewer", RES_CONTENT), Some(HashSet::new()));
        // Write-only grant still counts as readable.
        let wo = Grants { rows: vec![perm(9, "write")] };
        assert!(wo.visible_ids("viewer", RES_CONTENT).unwrap().contains(&9));
    }

    #[test]
    fn invalid_level_string_is_inert() {
        let g = Grants { rows: vec![perm(7, "superuser")] };
        assert!(!g.can("viewer", Action::Read, RES_CONTENT, 7));
    }
}
