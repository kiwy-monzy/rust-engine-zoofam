//! Async client for the TRC TICIDIS SGR ticketing API.
//!
//! Ported from `SGR/sgrapi/__init__.py`. Bodies/responses are `serde_json::Value`
//! to mirror the Python client's flexibility; typed helpers parse the common
//! results (stations, trips, train set).

use serde_json::Value;

use crate::sgr::models::{Station, TrainSet, Trip};

pub const BASE_URL: &str = "https://sgrticket-api.trc.co.tz/TICIDIS/api/v1/";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api returned status {0}")]
    Status(u16),
    #[error("unexpected response shape")]
    Shape,
}

pub type Result<T> = std::result::Result<T, Error>;

/// SGR API client. Holds an optional bearer token (from phone-OTP login).
#[derive(Clone)]
pub struct SgrClient {
    http: reqwest::Client,
    base: String,
    public: String,
    pub token: Option<String>,
    /// Optional signing headers (User-Agent / CustomHeaderParam / Content-MD5).
    pub user_agent: Option<String>,
    pub custom_header: Option<String>,
    pub content_md5: Option<String>,
}

impl SgrClient {
    pub fn new() -> Self {
        // The upstream API uses `verify=False`; accept invalid certs to match.
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_default();
        Self {
            http,
            base: BASE_URL.to_string(),
            public: format!("{BASE_URL}Public/"),
            token: None,
            user_agent: None,
            custom_header: None,
            content_md5: None,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    fn req(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut rb = rb;
        if let Some(t) = &self.token { rb = rb.bearer_auth(t); }
        if let Some(v) = &self.user_agent { rb = rb.header("User-Agent", v); }
        if let Some(v) = &self.custom_header { rb = rb.header("CustomHeaderParam", v); }
        if let Some(v) = &self.content_md5 { rb = rb.header("Content-MD5", v); }
        rb
    }

    async fn get(&self, url: &str) -> Result<Value> {
        let res = self.req(self.http.get(url)).send().await?;
        let code = res.status().as_u16();
        if code != 200 { return Err(Error::Status(code)); }
        Ok(res.json().await?)
    }

    async fn post(&self, url: &str, body: &Value) -> Result<Value> {
        let res = self.req(self.http.post(url)).json(body).send().await?;
        let code = res.status().as_u16();
        if code != 200 { return Err(Error::Status(code)); }
        Ok(res.json().await?)
    }

    // ── Reference data ────────────────────────────────────────────────────────

    /// `Public/GetTripAssignedStations` — raw boarding/landing stations response.
    pub async fn stations_raw(&self) -> Result<Value> {
        self.get(&format!("{}GetTripAssignedStations", self.public)).await
    }

    /// Typed convenience over [`stations_raw`] (drops unknown fields).
    pub async fn stations(&self) -> Result<Vec<Station>> {
        let v = self.stations_raw().await?;
        let data = v.get("data").unwrap_or(&v);
        Ok(serde_json::from_value(data.clone()).unwrap_or_default())
    }

    pub async fn configurations(&self) -> Result<Value> {
        self.get(&format!("{}DefaultConfiguration/DefaultConfiguration", self.base)).await
    }

    pub async fn nationalities(&self) -> Result<Value> {
        self.get(&format!("{}ConfigurationItem/NationalityTypesList", self.base)).await
    }

    // ── Schedules / trips ─────────────────────────────────────────────────────

    /// `Public/SearchTrip` — search schedules. `body` must contain
    /// boardingStationId, landingStationId, departureDate, passengerCount, …
    pub async fn search_trips_raw(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}SearchTrip", self.public), body).await
    }

    /// Convenience: search trips and parse the `data` array into [`Trip`]s.
    pub async fn search_trips(&self, body: &Value) -> Result<Vec<Trip>> {
        let v = self.search_trips_raw(body).await?;
        let data = v.get("data").cloned().unwrap_or(Value::Null);
        // The data shape varies; accept either an array or { trips: [...] }.
        let arr = match data {
            Value::Array(a) => a,
            Value::Object(ref o) => o.get("trips").and_then(|t| t.as_array()).cloned().unwrap_or_default(),
            _ => Vec::new(),
        };
        Ok(arr.into_iter().filter_map(|t| serde_json::from_value(t).ok()).collect())
    }

    // ── Seats ─────────────────────────────────────────────────────────────────

    /// `Public/TrainSetBy` — available seats for a chosen trip/train set.
    pub async fn train_set_raw(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}TrainSetBy", self.public), body).await
    }

    /// Convenience: fetch the train set and parse coaches + seats.
    pub async fn train_set(&self, body: &Value) -> Result<TrainSet> {
        let v = self.train_set_raw(body).await?;
        TrainSet::from_api_value(&v).ok_or(Error::Shape)
    }

    // ── Pricing ───────────────────────────────────────────────────────────────

    pub async fn ticket_types_by_route(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}RouteStationPrice/GetTicketTypeListByRoute", self.base), body).await
    }

    pub async fn price(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}RouteStationPrice/GetPrice", self.base), body).await
    }

    // ── Phone OTP login ───────────────────────────────────────────────────────

    pub async fn send_otp(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}User/PublicPhoneLoginSendSms", self.base), body).await
    }

    /// Returns the login response (carries the access token on success).
    pub async fn verify_otp(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}User/PublicPhoneLogin", self.base), body).await
    }

    // ── Booking ───────────────────────────────────────────────────────────────

    pub async fn create_booked_seat(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}Ticket/CreateBookedSeat", self.base), body).await
    }

    pub async fn create_ticket(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}Ticket/CreateTicket", self.base), body).await
    }

    pub async fn create_ticket_bill(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}Ticket/CreateTicketBill", self.base), body).await
    }

    // ── Payment (GEPG) ────────────────────────────────────────────────────────

    pub async fn active_payment_providers(&self) -> Result<Value> {
        self.get(&format!("{}Gepg/GetActivePaymentProviders", self.base)).await
    }

    pub async fn push_to_pay_operators(&self) -> Result<Value> {
        self.get(&format!("{}Gepg/PushToPay/Operators", self.base)).await
    }

    pub async fn control_no_by_bill(&self, bill_id: &str) -> Result<Value> {
        self.get(&format!("{}Ticket/GetControlNumberByBilldId/{bill_id}", self.base)).await
    }

    pub async fn push_to_pay(&self, body: &Value) -> Result<Value> {
        self.post(&format!("{}Gepg/PushToPay", self.base), body).await
    }

    pub async fn tickets_by_user(&self) -> Result<Value> {
        self.get(&format!("{}Ticket/GetTicketsByUser", self.base)).await
    }
}

impl Default for SgrClient {
    fn default() -> Self { Self::new() }
}
