use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use db::{conn, DbConn, DbPool};
use models::schema::{gateway_roles, gateway_user_roles, gateway_users, support_tickets};
use models::{CreateTicket, NewTicket, SupportTicket, TicketView, TICKET_CATEGORIES};

use crate::{Error, Result};

fn is_admin(c: &mut DbConn, user_id: &str) -> Result<bool> {
    let found: i64 = gateway_user_roles::table
        .inner_join(gateway_roles::table)
        .filter(gateway_user_roles::user_id.eq(user_id))
        .filter(gateway_roles::name.eq("admin"))
        .count()
        .get_result(c)?;
    Ok(found > 0)
}

fn display_label(c: &mut DbConn, user_id: &str) -> Result<String> {
    let (display, email): (String, String) = gateway_users::table
        .find(user_id)
        .select((gateway_users::display_name, gateway_users::email))
        .first(c)
        .map_err(|_| Error::NotFound("user"))?;
    Ok(if display.is_empty() { email } else { display })
}

/// A user sees tickets they sent, tickets addressed to them, and — when they
/// hold the admin role — every role-addressed ticket (recipient IS NULL).
/// `admin_to_admin` stays admin-only even for its sender.
pub fn list(pool: &DbPool, user_id: &str) -> Result<Vec<TicketView>> {
    let mut c = conn(pool)?;
    let admin = is_admin(&mut c, user_id)?;

    let tickets: Vec<SupportTicket> = if admin {
        support_tickets::table
            .filter(
                support_tickets::created_by
                    .eq(user_id)
                    .or(support_tickets::assigned_to.eq(user_id))
                    .or(support_tickets::assigned_to.is_null()),
            )
            .order(support_tickets::updated_at.desc())
            .load(&mut c)?
    } else {
        support_tickets::table
            .filter(
                support_tickets::created_by
                    .eq(user_id)
                    .or(support_tickets::assigned_to.eq(user_id)),
            )
            .order(support_tickets::updated_at.desc())
            .load(&mut c)?
    };

    let mut out = Vec::with_capacity(tickets.len());
    for t in tickets {
        if t.category == "admin_to_admin" && !admin && t.created_by != user_id {
            continue;
        }
        let sender = display_label(&mut c, &t.created_by)?;
        let recipient = match &t.assigned_to {
            Some(id) => Some(display_label(&mut c, id)?),
            None => None,
        };
        out.push(TicketView {
            sender,
            recipient,
            ticket: t,
        });
    }
    Ok(out)
}

pub fn create(pool: &DbPool, sender_id: &str, input: CreateTicket) -> Result<SupportTicket> {
    if !TICKET_CATEGORIES.contains(&input.category.as_str()) {
        return Err(Error::Invalid(
            "the category must be viewer_to_admin, admin_to_admin or viewer_to_viewer".into(),
        ));
    }
    let subject = input.subject.trim().to_string();
    if subject.is_empty() || subject.chars().count() > 200 {
        return Err(Error::Invalid(
            "the subject must be 1-200 characters".into(),
        ));
    }
    let body = input.body.trim().to_string();
    if body.is_empty() {
        return Err(Error::Invalid("the ticket needs a message".into()));
    }

    let mut c = conn(pool)?;
    let sender_admin = is_admin(&mut c, sender_id)?;
    let assigned_to = match input.category.as_str() {
        "admin_to_admin" => {
            if !sender_admin {
                return Err(Error::Forbidden(
                    "only admins may open admin_to_admin tickets".into(),
                ));
            }
            None
        }
        "viewer_to_admin" => {
            if sender_admin {
                return Err(Error::Invalid(
                    "admins reach other admins via admin_to_admin".into(),
                ));
            }
            None
        }
        _ => {
            let rid = input
                .recipient_id
                .as_deref()
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .ok_or_else(|| {
                    Error::Invalid("viewer_to_viewer tickets need a recipient".into())
                })?;
            if rid == sender_id {
                return Err(Error::Invalid(
                    "you cannot send a ticket to yourself".into(),
                ));
            }
            gateway_users::table
                .find(rid)
                .select(gateway_users::id)
                .first::<String>(&mut c)
                .map_err(|_| Error::NotFound("recipient"))?;
            Some(rid.to_string())
        }
    };

    let now = Utc::now().naive_utc();
    let record = NewTicket {
        id: Uuid::new_v4().to_string(),
        subject,
        description: body,
        status: "open".into(),
        priority: "normal".into(),
        category: input.category,
        created_by: sender_id.to_string(),
        assigned_to,
        created_at: now,
        updated_at: now,
        closed_at: None,
    };
    let ticket_id = record.id.clone();
    diesel::insert_into(support_tickets::table)
        .values(&record)
        .execute(&mut c)
        .map_err(Into::<crate::Error>::into)?;
    support_tickets::table
        .find(&ticket_id)
        .select(SupportTicket::as_select())
        .first(&mut c)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Error::NotFound("ticket after insert"),
            other => other.into(),
        })
}

/// Sender, named recipient, or an admin (for role-addressed tickets) may close.
pub fn close(pool: &DbPool, user_id: &str, admin: bool, ticket_id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let ticket: SupportTicket = support_tickets::table
        .find(ticket_id)
        .select(SupportTicket::as_select())
        .first(&mut c)
        .map_err(|_| Error::NotFound("ticket"))?;

    let allowed = ticket.created_by == user_id
        || ticket.assigned_to.as_deref() == Some(user_id)
        || (admin && ticket.assigned_to.is_none());
    if !allowed {
        return Err(Error::Forbidden("you may not close this ticket".into()));
    }

    diesel::update(support_tickets::table.find(ticket_id))
        .set((
            support_tickets::status.eq("closed"),
            support_tickets::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut c)?;
    Ok(())
}

/// Minimal directory for the viewer_to_viewer recipient picker.
pub fn recipients(pool: &DbPool, exclude: &str) -> Result<Vec<(String, String)>> {
    let mut c = conn(pool)?;
    let rows: Vec<(String, String, String)> = gateway_users::table
        .filter(gateway_users::is_active.eq(true))
        .filter(gateway_users::id.ne(exclude))
        .select((
            gateway_users::id,
            gateway_users::display_name,
            gateway_users::email,
        ))
        .order(gateway_users::email_lower.asc())
        .load(&mut c)?;
    Ok(rows
        .into_iter()
        .map(|(id, display, email)| (id, if display.is_empty() { email } else { display }))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::users;

    fn pool() -> DbPool {
        let pool = db::create_pool_from(":memory:").expect("pool");
        db::run_migrations(&pool).expect("migrations");
        pool
    }

    fn user(pool: &DbPool, email: &str) -> String {
        let mut conn = pool.get().unwrap();
        users::create(
            &mut conn,
            models::CreateUser {
                email: email.into(),
                password: "password123".into(),
                display_name: email.split('@').next().unwrap().into(),
                avatar_url: String::new(),
                first_name: String::new(),
                middle_name: String::new(),
                last_name: String::new(),
                username: String::new(),
            },
        )
        .expect("user")
        .id
    }

    fn role_id(pool: &DbPool, name: &str) -> i32 {
        crate::roles::list(pool)
            .unwrap()
            .into_iter()
            .find(|r| r.role.name == name)
            .unwrap()
            .role
            .id
    }

    fn ticket(
        pool: &DbPool,
        sender: &str,
        category: &str,
        recipient: Option<&str>,
    ) -> SupportTicket {
        create(
            pool,
            sender,
            CreateTicket {
                category: category.into(),
                recipient_id: recipient.map(str::to_string),
                subject: "help".into(),
                body: "please".into(),
            },
        )
        .expect("ticket")
    }

    #[test]
    fn viewer_to_admin_reaches_every_admin_but_not_other_viewers() {
        let pool = pool();
        let mut conn = pool.get().unwrap();
        let admin = user(&pool, "admin@example.com");
        let viewer = user(&pool, "viewer@example.com");
        let other = user(&pool, "other@example.com");
        users::assign_role(&mut conn, &admin, role_id(&pool, "admin")).unwrap();
        users::assign_role(&mut conn, &viewer, role_id(&pool, "viewer")).unwrap();

        let t = ticket(&pool, &viewer, "viewer_to_admin", None);

        let seen_admin: Vec<String> = list(&pool, &admin)
            .unwrap()
            .iter()
            .map(|v| v.ticket.id.clone())
            .collect();
        assert!(seen_admin.contains(&t.id));

        let seen_other: Vec<String> = list(&pool, &other)
            .unwrap()
            .iter()
            .map(|v| v.ticket.id.clone())
            .collect();
        assert!(!seen_other.contains(&t.id));

        assert!(close(&pool, &admin, true, &t.id).is_ok());
        assert_eq!(list(&pool, &viewer).unwrap()[0].ticket.status, "closed");
    }

    #[test]
    fn admin_to_admin_stays_between_admins() {
        let pool = pool();
        let mut conn = pool.get().unwrap();
        let admin = user(&pool, "admin2@example.com");
        let viewer = user(&pool, "viewer2@example.com");
        users::assign_role(&mut conn, &admin, role_id(&pool, "admin")).unwrap();
        users::assign_role(&mut conn, &viewer, role_id(&pool, "viewer")).unwrap();

        let t = ticket(&pool, &admin, "admin_to_admin", None);
        assert!(list(&pool, &admin)
            .unwrap()
            .iter()
            .any(|v| v.ticket.id == t.id));
        assert!(!list(&pool, &viewer)
            .unwrap()
            .iter()
            .any(|v| v.ticket.id == t.id));

        assert!(create(
            &pool,
            &viewer,
            CreateTicket {
                category: "admin_to_admin".into(),
                recipient_id: None,
                subject: "s".into(),
                body: "b".into()
            }
        )
        .is_err());
    }

    #[test]
    fn viewer_to_viewer_is_private_between_the_pair() {
        let pool = pool();
        let alice = user(&pool, "alice@example.com");
        let bob = user(&pool, "bob@example.com");
        let mallory = user(&pool, "mallory@example.com");

        let t = ticket(&pool, &alice, "viewer_to_viewer", Some(&bob));

        assert!(list(&pool, &bob)
            .unwrap()
            .iter()
            .any(|v| v.ticket.id == t.id));
        assert!(!list(&pool, &mallory)
            .unwrap()
            .iter()
            .any(|v| v.ticket.id == t.id));
        assert!(close(&pool, &mallory, false, &t.id).is_err());
        assert!(close(&pool, &bob, false, &t.id).is_ok());
        assert!(create(
            &pool,
            &alice,
            CreateTicket {
                category: "viewer_to_viewer".into(),
                recipient_id: Some(alice.clone()),
                subject: "s".into(),
                body: "b".into()
            }
        )
        .is_err());
    }

    #[test]
    fn admins_cannot_use_the_viewer_to_admin_lane() {
        let pool = pool();
        let mut conn = pool.get().unwrap();
        let admin = user(&pool, "admin3@example.com");
        users::assign_role(&mut conn, &admin, role_id(&pool, "admin")).unwrap();
        assert!(create(
            &pool,
            &admin,
            CreateTicket {
                category: "viewer_to_admin".into(),
                recipient_id: None,
                subject: "s".into(),
                body: "b".into()
            }
        )
        .is_err());
    }
}
