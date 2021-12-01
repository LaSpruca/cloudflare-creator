mod cf_check;
mod create_binary;
pub mod error;
mod ssh;

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex},
};

use actix_rt::time::Instant;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use create_binary::{compile_source, create_source_file};
use error::Error;
use serde::{Deserialize, Serialize};
use ssh::{create_cron_job, create_session, upload_file};

use crate::cf_check::check_cf;

#[derive(Serialize, Clone)]
enum JobStatus {
    Submitted,
    CheckingToken,
    BuildingScript,
    Uploading,
    CreatingCronJob,
    Done(String),
    Error(Error),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateJobRequest {
    cf_token: String,
    cf_email: String,
    cf_zone: String,
    cf_dns: String,

    ssh_address: String,
    ssh_port: usize,
    ssh_username: String,
    ssh_ras_key: Option<String>,
    ssh_password: Option<String>,
}

#[derive(Serialize)]
struct CreateJobResponse {
    id: u128,
}

#[derive(Serialize)]
struct JobStatusResponse {
    status: JobStatus,
}

type JobStatusMap = HashMap<u128, Arc<Mutex<JobStatus>>>;

#[get("/status/{id}")]
fn job_status(id: web::Path<u128>, data: web::Data<Mutex<JobStatusMap>>) -> HttpResponse {
    let mut data_lock = data.lock().unwrap();
    let mut remove = false;
    let response = match data_lock.entry(*id) {
        Entry::Occupied(status) => {
            let status = status.get().lock().unwrap();

            let response = JobStatusResponse {
                status: (*status).clone(),
            };

            match *status {
                JobStatus::Done(_) | JobStatus::Error(_) => remove = true,
                _ => {}
            }

            HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
        }
        Entry::Vacant(_) => HttpResponse::NotFound()
            .body(&format!("{{\'error\': \'No job with id {} exists\'}}", id)),
    };

    if remove {
        data_lock.remove_entry(&*id);
    }

    response
}

#[post("/create-job")]
fn create_job(
    payload: web::Json<CreateJobRequest>,
    data: web::Data<Mutex<JobStatusMap>>,
    start: web::Data<Instant>,
) -> HttpResponse {
    let job_id = Instant::now().duration_since(**start);
    let status = Arc::new(Mutex::new(JobStatus::Submitted));
    let mut data_lock = data.lock().unwrap();
    data_lock.insert(job_id.as_millis(), status.clone());
    actix_rt::spawn(async move {
        *status.lock().unwrap() = JobStatus::CheckingToken;

        match check_cf(&payload.cf_token, &payload.cf_email, &payload.cf_zone).await {
            Ok(_) => {}
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        };

        *status.lock().unwrap() = JobStatus::BuildingScript;

        let filename = match create_source_file(
            payload.cf_token.clone(),
            payload.cf_zone.clone(),
            payload.cf_dns.clone(),
            payload.cf_email.clone(),
        ) {
            Ok(a) => a,
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        };

        let compiled = match compile_source(&filename) {
            Ok(a) => a,
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        };

        *status.lock().unwrap() = JobStatus::Uploading;

        let sess = match create_session(
            payload.ssh_address.clone(),
            payload.ssh_port.clone(),
            payload.ssh_username.clone(),
            payload.ssh_password.clone(),
            payload.ssh_ras_key.clone(),
        ) {
            Ok(a) => a,
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        };

        match upload_file(&compiled, &sess) {
            Ok(a) => a,
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        };

        *status.lock().unwrap() = JobStatus::CreatingCronJob;

        match create_cron_job(&filename, &sess) {
            Ok(a) => a,
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        }

        *status.lock().unwrap() = JobStatus::Done(filename);
    });
    let response = CreateJobResponse {
        id: job_id.as_millis(),
    };
    HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(job_status)
            .service(create_job)
            .data(Mutex::new(JobStatusMap::new()))
            .data(Instant::now())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
