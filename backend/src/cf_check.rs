use crate::error::{Error, ErrorKind};
use actix_web::{client::Client, http::Method};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct CfErrorChain {
    code: i32,
    message: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct CfError {
    code: i32,
    message: String,
    error_chain: Vec<CfErrorChain>,
}

#[derive(Deserialize, Debug)]
struct CfResponse {
    success: bool,
    errors: Vec<CfError>,
}

#[derive(Deserialize, Debug)]
struct Zone {
    name: String,
    id: String,
    permissions: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct CfZones {
    result: Vec<Zone>,
}

const API_BASE: &str = "https://api.cloudflare.com/client/v4";

pub async fn check_cf(token: &str, email: &str, zone: &str) -> Result<(), Error> {
    let client = Client::default();

    let mut request = match client
        .request(Method::GET, &format!("{}/zones", API_BASE))
        .set_header("X-Auth-Email", email)
        .set_header("Authorization", format!("Bearer {}", token))
        .set_header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(a) => a,
        Err(e) => {
            println!("{}", e.to_string());
            return Err(Error {
                kind: crate::error::ErrorKind::CfError,
                message: "Error getting information".into(),
            });
        }
    };

    let body = &request.body().await.unwrap().to_vec();
    let res_str = String::from_utf8(body.to_owned()).unwrap();
    let res: CfResponse = serde_json::from_str(&res_str).unwrap();

    if !res.success {
        return Err(Error {
            kind: ErrorKind::CfError,
            message: serde_json::to_string(&res.errors).unwrap(),
        });
    }

    let zones: CfZones = serde_json::from_str(&res_str).unwrap();

    let zone_id = match zones.result.iter().find(|f| f.name == zone) {
        Some(a) => a,
        None => {
            return Err(Error {
                kind: ErrorKind::CfError,
                message: format!("Token does not have any zone called {}", zone),
            })
        }
    };

    if !zone_id.permissions.iter().any(|f| f == "#dns_records:edit")
        || !zone_id.permissions.iter().any(|f| f == "#dns_records:edit")
    {
        return Err(Error {
            kind: ErrorKind::CfError,
            message: format!("Token does not have read or edit access to zone {}", zone),
        });
    }

    Ok(())
}
