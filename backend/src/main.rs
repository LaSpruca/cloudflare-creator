mod cf_check;
mod create_binary;
pub mod error;
mod ssh;

use crate::cf_check::check_cf;
use actix_cors::Cors;
use actix_rt::time::Instant;
use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use create_binary::{compile_source, create_source_file};
use error::Error;
use serde::{Deserialize, Serialize};
use ssh::{create_cron_job, create_session, upload_file};
use std::fs::remove_file;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex},
};
use log::info;

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
    id: u64,
}

#[derive(Serialize)]
struct JobStatusResponse {
    status: JobStatus,
}

type JobStatusMap = HashMap<u64, Arc<Mutex<JobStatus>>>;

#[get("/status/{id}")]
fn get_job_status(id: web::Path<u64>, data: web::Data<Arc<Mutex<JobStatusMap>>>) -> HttpResponse {
    info!("Got request");
    let mut data_lock = data.lock().unwrap();
    info!("Got data_lock");
    let mut remove = false;
    let response = match data_lock.entry(*id) {
        Entry::Occupied(status) => {
            let status = status.get().lock().unwrap();
            info!("Get status lock");

            let response = JobStatusResponse {
                status: (*status).clone(),
            };

            match *status {
                JobStatus::Done(_) | JobStatus::Error(_) => remove = true,
                _ => {}
            }

            HttpResponse::Ok()
                .content_type("application/json")
                .body(serde_json::to_string(&response).unwrap())
        }
        Entry::Vacant(_) => HttpResponse::NotFound()
            .content_type("application/json")
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
    data: web::Data<Arc<Mutex<JobStatusMap>>>,
    start: web::Data<Instant>,
) -> HttpResponse {
    let job_id = Instant::now().duration_since(**start);
    let job_id = job_id.as_millis() as u64;
    let status = Arc::new(Mutex::new(JobStatus::Submitted));
    let mut data_lock = data.lock().unwrap();
    data_lock.insert(job_id, status.clone());
    drop(data_lock);
    actix_rt::spawn(async move {
        {
            let mut lock = status.lock().unwrap();
            *lock = JobStatus::CheckingToken;
        }

        match check_cf(&payload.cf_token, &payload.cf_email, &payload.cf_zone).await {
            Ok(_) => {}
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        };

        {
            let mut lock = status.lock().unwrap();
            *lock = JobStatus::BuildingScript;
        }

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

        {
            let mut lock = status.lock().unwrap();
            *lock = JobStatus::Uploading;
        }

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
                remove_file(&compiled).unwrap_or(());
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

        remove_file(&compiled).unwrap_or(());

        {
            let mut lock = status.lock().unwrap();
            *lock = JobStatus::CreatingCronJob;
        }

        match create_cron_job(&compiled, &sess) {
            Ok(a) => a,
            Err(e) => {
                *status.lock().unwrap() = JobStatus::Error(e);
                return;
            }
        }

        *status.lock().unwrap() = JobStatus::Done(compiled);
    });
    let response = CreateJobResponse { id: job_id };
    HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&response).unwrap())
}

#[get("/")]
async fn index() -> impl Responder {
    "Yes this is the Cloudflare Updater backend, good job you figured it out"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let start = Instant::now();
    let app_state = Arc::new(Mutex::new(JobStatusMap::new()));
    HttpServer::new(move || {
        #[cfg(debug_assertions)]
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();

        #[cfg(not(debug_assertions))]
        let cors = Cors::default()
            .allowed_origin("https://cf-update.laspruca.nz")
            .allow_any_header()
            .allowed_methods(vec!["GET", "POST"]);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(get_job_status)
            .service(index)
            .service(create_job)
            .data(app_state.clone())
            .data(start)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
