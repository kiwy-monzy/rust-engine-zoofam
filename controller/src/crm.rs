use chrono::NaiveDateTime;
use db::DbPool;
use diesel::prelude::*;
use uuid::Uuid;
use crate::Error;
use models::crm::*;
use models::schema::*;

fn nid()->String{Uuid::new_v4().to_string()}
fn now()->NaiveDateTime{ chrono::Utc::now().naive_utc() }

pub fn leads(pool:&DbPool)->Result<Vec<Lead>,Error>{ let mut c=db::conn(pool)?; Ok(crm_leads::table.order(crm_leads::created_at.desc()).load(&mut c)?) }
pub fn lead_create(pool:&DbPool, mut v:NewLead)->Result<Lead,Error>{ if v.id.is_empty(){v.id=nid();} if v.lead_number.is_empty(){v.lead_number=format!("LD-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(crm_leads::table).values(&v).execute(&mut c)?; Ok(crm_leads::table.find(&v.id).first(&mut c)?) }
pub fn lead_update(pool:&DbPool, id:&str, status:&str)->Result<Lead,Error>{ let mut c=db::conn(pool)?; diesel::update(crm_leads::table.find(id)).set((crm_leads::status.eq(status), crm_leads::updated_at.eq(now()))).execute(&mut c)?; Ok(crm_leads::table.find(id).first(&mut c)?) }

pub fn opportunities(pool:&DbPool)->Result<Vec<Opportunity>,Error>{ let mut c=db::conn(pool)?; Ok(crm_opportunities::table.order(crm_opportunities::created_at.desc()).load(&mut c)?) }
pub fn opp_create(pool:&DbPool, mut v:NewOpportunity)->Result<Opportunity,Error>{ if v.id.is_empty(){v.id=nid();} if v.opp_number.is_empty(){v.opp_number=format!("OPP-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(crm_opportunities::table).values(&v).execute(&mut c)?; Ok(crm_opportunities::table.find(&v.id).first(&mut c)?) }

pub fn customers(pool:&DbPool)->Result<Vec<Customer>,Error>{ let mut c=db::conn(pool)?; Ok(crm_customers::table.order(crm_customers::created_at.desc()).load(&mut c)?) }
pub fn customer_create(pool:&DbPool, mut v:NewCustomer)->Result<Customer,Error>{ if v.id.is_empty(){v.id=nid();} if v.customer_number.is_empty(){v.customer_number=format!("CUS-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(crm_customers::table).values(&v).execute(&mut c)?; Ok(crm_customers::table.find(&v.id).first(&mut c)?) }

pub fn contacts(pool:&DbPool)->Result<Vec<Contact>,Error>{ let mut c=db::conn(pool)?; Ok(crm_contacts::table.load(&mut c)?) }
pub fn contact_create(pool:&DbPool, mut v:NewContact)->Result<Contact,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(crm_contacts::table).values(&v).execute(&mut c)?; Ok(crm_contacts::table.find(&v.id).first(&mut c)?) }

pub fn activities(pool:&DbPool)->Result<Vec<Activity>,Error>{ let mut c=db::conn(pool)?; Ok(crm_activities::table.order(crm_activities::created_at.desc()).load(&mut c)?) }
pub fn activity_create(pool:&DbPool, mut v:NewActivity)->Result<Activity,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(crm_activities::table).values(&v).execute(&mut c)?; Ok(crm_activities::table.find(&v.id).first(&mut c)?) }

pub fn quotes(pool:&DbPool)->Result<Vec<Quote>,Error>{ let mut c=db::conn(pool)?; Ok(crm_quotes::table.order(crm_quotes::created_at.desc()).load(&mut c)?) }
pub fn quote_create(pool:&DbPool, mut v:NewQuote)->Result<Quote,Error>{ if v.id.is_empty(){v.id=nid();} if v.quote_number.is_empty(){v.quote_number=format!("QT-{}", &v.id[..8]);} let mut c=db::conn(pool)?; diesel::insert_into(crm_quotes::table).values(&v).execute(&mut c)?; Ok(crm_quotes::table.find(&v.id).first(&mut c)?) }
pub fn quote_lines(pool:&DbPool, qid:&str)->Result<Vec<QuoteLine>,Error>{ let mut c=db::conn(pool)?; Ok(crm_quote_lines::table.filter(crm_quote_lines::quote_id.eq(qid)).load(&mut c)?) }
pub fn quote_line_create(pool:&DbPool, mut v:NewQuoteLine)->Result<QuoteLine,Error>{ if v.id.is_empty(){v.id=nid();} let mut c=db::conn(pool)?; diesel::insert_into(crm_quote_lines::table).values(&v).execute(&mut c)?; Ok(crm_quote_lines::table.find(&v.id).first(&mut c)?) }

/// Accept quote → create sales order + lines (ERP Sales) and link. Returns created sales order id.
pub fn quote_accept(pool:&DbPool, quote_id:&str)->Result<models::erp::SalesOrder,Error>{
    let mut c=db::conn(pool)?; 
    // load quote + lines
    let q: Quote = crm_quotes::table.find(quote_id).first(&mut c)?;
    if q.sales_order_id.is_some(){ return Err(Error::Conflict("quote already accepted".into())); }
    let lines: Vec<QuoteLine> = crm_quote_lines::table.filter(crm_quote_lines::quote_id.eq(quote_id)).load(&mut c)?;
    if lines.is_empty(){ return Err(Error::Invalid("quote has no lines".into())); }
    // resolve customer name
    let cust_name = if let Some(cid)= &q.customer_id { let cust: Customer = crm_customers::table.find(cid).first(&mut c)?; cust.name } else { "CRM Customer".into() };
    let total = q.total.unwrap_or(0.0);
    let so_id: String = c.transaction(|conn|{
        let sid=nid(); let so_num=format!("SO-{}", &sid[..8]);
        diesel::insert_into(commerce_sales_orders::table).values((
            commerce_sales_orders::id.eq(&sid),
            commerce_sales_orders::so_number.eq(&so_num),
            commerce_sales_orders::customer_name.eq(&cust_name),
            commerce_sales_orders::status.eq("DRAFT"),
        )).execute(conn)?;
        for l in &lines {
            diesel::insert_into(commerce_sales_lines::table).values((
                commerce_sales_lines::id.eq(nid()),
                commerce_sales_lines::sales_order_id.eq(&sid),
                commerce_sales_lines::product_id.eq(&l.product_id),
                commerce_sales_lines::quantity.eq(l.quantity),
                commerce_sales_lines::unit_price.eq(l.unit_price),
            )).execute(conn)?;
        }
        diesel::update(crm_quotes::table.find(quote_id)).set((crm_quotes::sales_order_id.eq(&sid), crm_quotes::status.eq("ACCEPTED"))).execute(conn)?;
        // also create invoice draft for accounting
        let inv_id=nid(); diesel::insert_into(accounting_invoices::table).values((
            accounting_invoices::id.eq(&inv_id),
            accounting_invoices::invoice_number.eq(format!("INV-{}", &inv_id[..8])),
            accounting_invoices::sales_order_id.eq(&sid),
            accounting_invoices::quote_id.eq(quote_id),
            accounting_invoices::total.eq(total),
            accounting_invoices::status.eq("DRAFT"),
        )).execute(conn)?;
        Ok::<String, diesel::result::Error>(sid)
    })?;
    let mut c2=db::conn(pool)?; let so: models::erp::SalesOrder = commerce_sales_orders::table.find(&so_id).first(&mut c2)?; Ok(so)
}
