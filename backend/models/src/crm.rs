use crate::schema::*;
use chrono::{DateTime, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

fn de_opt_naive<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(d)?;
    match opt {
        None => Ok(None),
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.naive_utc()));
            }
            for fmt in [
                "%Y-%m-%dT%H:%M:%S%.f",
                "%Y-%m-%dT%H:%M:%S",
                "%Y-%m-%d %H:%M:%S%.f",
                "%Y-%m-%d %H:%M:%S",
                "%Y-%m-%d",
            ] {
                if let Ok(nd) = NaiveDateTime::parse_from_str(&s, fmt) {
                    return Ok(Some(nd));
                }
            }
            let t = s.trim_end_matches('Z');
            for fmt in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S"] {
                if let Ok(nd) = NaiveDateTime::parse_from_str(t, fmt) {
                    return Ok(Some(nd));
                }
            }
            Err(serde::de::Error::custom(format!(
                "invalid datetime '{}'",
                s
            )))
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = crm_leads)]
pub struct Lead {
    pub id: String,
    pub lead_number: String,
    pub company_name: Option<String>,
    pub contact_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub assigned_to: Option<String>,
    pub product_id: Option<String>,
    pub estimated_value: Option<f32>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crm_leads)]
pub struct NewLead {
    pub id: String,
    pub lead_number: String,
    pub company_name: Option<String>,
    pub contact_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub assigned_to: Option<String>,
    pub product_id: Option<String>,
    pub estimated_value: Option<f32>,
    pub notes: Option<String>,
}
#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = crm_leads)]
pub struct UpdateLead {
    pub company_name: Option<String>,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub product_id: Option<String>,
    pub estimated_value: Option<f32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = crm_opportunities)]
pub struct Opportunity {
    pub id: String,
    pub opp_number: String,
    pub lead_id: Option<String>,
    pub customer_id: Option<String>,
    pub stage: String,
    pub probability: Option<f32>,
    pub amount: Option<f32>,
    pub expected_close: Option<NaiveDateTime>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crm_opportunities)]
pub struct NewOpportunity {
    pub id: String,
    pub opp_number: String,
    pub lead_id: Option<String>,
    pub customer_id: Option<String>,
    pub stage: String,
    pub probability: Option<f32>,
    pub amount: Option<f32>,
    #[serde(default, deserialize_with = "de_opt_naive")]
    pub expected_close: Option<NaiveDateTime>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = crm_customers)]
pub struct Customer {
    pub id: String,
    pub customer_number: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub lead_id: Option<String>,
    pub opportunity_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crm_customers)]
pub struct NewCustomer {
    pub id: String,
    pub customer_number: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub lead_id: Option<String>,
    pub opportunity_id: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = crm_contacts)]
pub struct Contact {
    pub id: String,
    pub customer_id: String,
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub phone: Option<String>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crm_contacts)]
pub struct NewContact {
    pub id: String,
    pub customer_id: String,
    pub name: String,
    pub email: Option<String>,
    pub role: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = crm_activities)]
pub struct Activity {
    pub id: String,
    pub related_type: String,
    pub related_id: String,
    pub activity_type: String,
    pub subject: String,
    pub due_at: Option<NaiveDateTime>,
    pub done: i32,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crm_activities)]
pub struct NewActivity {
    pub id: String,
    pub related_type: String,
    pub related_id: String,
    pub activity_type: String,
    pub subject: String,
    #[serde(default, deserialize_with = "de_opt_naive")]
    pub due_at: Option<NaiveDateTime>,
    pub done: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = crm_quotes)]
pub struct Quote {
    pub id: String,
    pub quote_number: String,
    pub customer_id: Option<String>,
    pub opportunity_id: Option<String>,
    pub status: String,
    pub valid_until: Option<NaiveDateTime>,
    pub total: Option<f32>,
    pub sales_order_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crm_quotes)]
pub struct NewQuote {
    pub id: String,
    pub quote_number: String,
    pub customer_id: Option<String>,
    pub opportunity_id: Option<String>,
    pub status: String,
    #[serde(default, deserialize_with = "de_opt_naive")]
    pub valid_until: Option<NaiveDateTime>,
    pub total: Option<f32>,
    pub sales_order_id: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = crm_quote_lines)]
pub struct QuoteLine {
    pub id: String,
    pub quote_id: String,
    pub product_id: String,
    pub quantity: f32,
    pub unit_price: f32,
    pub notes: Option<String>,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crm_quote_lines)]
pub struct NewQuoteLine {
    pub id: String,
    pub quote_id: String,
    pub product_id: String,
    pub quantity: f32,
    pub unit_price: f32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = accounting_invoices)]
pub struct Invoice {
    pub id: String,
    pub invoice_number: String,
    pub sales_order_id: Option<String>,
    pub quote_id: Option<String>,
    pub total: f32,
    pub status: String,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = accounting_invoices)]
pub struct NewInvoice {
    pub id: String,
    pub invoice_number: String,
    pub sales_order_id: Option<String>,
    pub quote_id: Option<String>,
    pub total: f32,
    pub status: String,
}
