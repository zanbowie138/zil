//! Currencies: the common codes known offline, and exchange rates loaded once from cache or open.er-api.com.

use super::{MONEY, REGISTRY, UnitDef};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    pub static RATES_LOADED: RefCell<bool> = const { RefCell::new(false) };
    pub static RATES_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Common currencies, known without fetching, so `25 USD/h * 40 h` stays offline.
pub const CURRENCIES: &str = "AUD BGN BRL CAD CHF CNY CZK DKK EUR GBP HKD HUF IDR ILS INR ISK JPY KRW MXN MYR NOK NZD PHP PLN RON SEK SGD THB TRY USD ZAR";

pub fn currency(code: &str, scale: f64) -> Rc<UnitDef> {
    Rc::new(UnitDef { name: code.into(), scale: Cell::new(scale), offset: 0.0, dim: MONEY })
}

pub fn rate_error() -> String {
    RATES_ERROR.with(|e| format!("cannot load currency rates: {}", e.borrow()))
}

/// Fills in every currency's rate, once; on failure their scales stay NaN and `rate_error` says why.
pub fn load_rates_once() {
    if RATES_LOADED.with(|l| l.replace(true)) {
        return;
    }
    RATES_ERROR.with(|e| e.borrow_mut().clear());
    match load_rates() {
        Ok(rates) => REGISTRY.with(|r| {
            let mut r = r.borrow_mut();
            for (code, per_eur) in rates {
                match r.get(&code) {
                    Some(def) => def.scale.set(1.0 / per_eur),
                    None => {
                        let def = currency(&code, 1.0 / per_eur);
                        r.entry(code.to_lowercase()).or_insert(def.clone());
                        r.insert(code, def);
                    }
                }
            }
        }),
        Err(e) => RATES_ERROR.with(|x| *x.borrow_mut() = e),
    }
}

/// Lets the next currency conversion try loading rates again, now that it may go online.
pub fn retry_rates() {
    RATES_LOADED.with(|l| *l.borrow_mut() = false);
}

fn fetch_rates() -> Result<String, String> {
    if !crate::modules::sys::network_allowed() {
        return Err("not cached; call allow_network_access() to fetch them".into());
    }
    crate::modules::sys::get("https://open.er-api.com/v6/latest/EUR", &[])
}

/// Units per 1 EUR from open.er-api.com (about 160 currencies, updated daily), cached on disk for a day.
fn load_rates() -> Result<Vec<(String, f64)>, String> {
    let cache = crate::cache_dir().map(|d| d.join("er-rates.json"));
    let age = cache.as_ref().and_then(|p| p.metadata().ok()?.modified().ok()?.elapsed().ok());
    let body = match &cache {
        Some(p) if age.is_some_and(|a| a.as_secs() < 24 * 3600) => std::fs::read_to_string(p).map_err(|e| e.to_string())?,
        _ => match fetch_rates() {
            Ok(body) => {
                if let Some(p) = &cache {
                    let _ = std::fs::create_dir_all(p.parent().unwrap());
                    let _ = std::fs::write(p, &body);
                }
                body
            }
            // Offline or not allowed online: fall back to a stale cache if there is one.
            Err(e) => match cache.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
                Some(body) => {
                    let days = age.map_or(0, |a| a.as_secs() / 86400);
                    eprintln!("note: using currency rates cached {days} day{} ago", if days == 1 { "" } else { "s" });
                    body
                }
                None => return Err(e),
            },
        },
    };
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
    let mut rates: Vec<(String, f64)> =
        json["rates"].as_object().ok_or("unexpected response")?.iter().filter_map(|(k, v)| Some((k.clone(), v.as_f64()?))).collect();
    rates.push(("EUR".into(), 1.0));
    Ok(rates)
}
