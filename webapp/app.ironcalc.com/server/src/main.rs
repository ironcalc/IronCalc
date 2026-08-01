#[macro_use]
extern crate rocket;

mod database;
mod finance;
mod id;

use std::io::{self, BufWriter, Cursor, Write};
use std::sync::Mutex;

use database::{add_model, get_model_list_from_db, select_model, IronCalcDB};
use finance::cache::Cache;
use finance::provider::FinanceProvider;
use finance::yahoo::YahooFinanceProvider;
use ironcalc::base::finance::provider::FinanceError;
use ironcalc::base::task::FinanceFetchTask;
use ironcalc::base::Model as IModel;
use ironcalc::export::save_xlsx_to_writer;
use ironcalc::import::load_from_xlsx_bytes;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{ContentType, Header};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::State;

const MAX_SIZE_MB: u8 = 20;

use rocket_db_pools::{Connection, Database};

#[derive(Responder)]
struct FileResponder {
    inner: Vec<u8>,
    content_type: ContentType,
    disposition: Header<'static>,
}

/// Return an xlsx version of the app.
#[post("/api/download", data = "<data>")]
async fn download(data: Data<'_>) -> io::Result<FileResponder> {
    println!("Download xlsx");

    let bytes = data.open(MAX_SIZE_MB.megabytes()).into_bytes().await?;
    if !bytes.is_complete() {
        return Err(io::Error::other("The file was not fully uploaded"));
    };

    let model = IModel::from_bytes(&bytes, "en")
        .map_err(|e| io::Error::other(format!("Error creating model, '{e}'")))?;

    let mut buffer: Vec<u8> = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = BufWriter::new(cursor);
        save_xlsx_to_writer(&model, &mut writer)
            .map_err(|e| io::Error::other(format!("Error saving model: '{e}'")))?;
        writer.flush()?;
    }

    let content_type = ContentType::new(
        "application",
        "vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );

    let disposition = Header::new(
        "Content-Disposition".to_string(),
        "attachment; filename=\"data.xlsx\"".to_string(),
    );

    println!("Download: success. ");

    Ok(FileResponder {
        inner: buffer,
        content_type,
        disposition,
    })
}

/// Saves the model on a file called
#[post("/api/share", data = "<data>")]
async fn share(db: Connection<IronCalcDB>, data: Data<'_>) -> io::Result<String> {
    println!("start share");
    let hash = id::new_id();
    let bytes = data.open(MAX_SIZE_MB.megabytes()).into_bytes().await?;
    if !bytes.is_complete() {
        return Err(io::Error::other("file was not fully uploaded"));
    }
    add_model(db, &hash, &bytes).await?;
    println!("done share: '{}'", hash);
    Ok(hash)
}

#[get("/api/model/<hash>")]
async fn get_model(db: Connection<IronCalcDB>, hash: &str) -> io::Result<Option<Vec<u8>>> {
    let bytes = select_model(db, hash).await?;
    println!("Select model: '{}'", hash);
    Ok(bytes)
}

#[get("/api/list")]
async fn get_model_list(db: Connection<IronCalcDB>) -> io::Result<String> {
    let model_list = get_model_list_from_db(db).await?;
    println!("Model list: '{:?}'", model_list);
    Ok(model_list.join(","))
}

#[post("/api/upload/<name>", data = "<data>")]
async fn upload(data: Data<'_>, name: &str) -> io::Result<Vec<u8>> {
    println!("start upload");
    let bytes = data.open(MAX_SIZE_MB.megabytes()).into_bytes().await?;
    if !bytes.is_complete() {
        return Err(io::Error::other("file was not fully uploaded"));
    }
    let workbook = load_from_xlsx_bytes(&bytes, name.trim_end_matches(".xlsx"), "en", "UTC")
        .map_err(|e| io::Error::other(format!("Error loading model: '{e}'")))?;
    let model = IModel::from_workbook(workbook, "en")
        .map_err(|e| io::Error::other(format!("Error creating model: '{e}'")))?;
    println!("end upload");
    Ok(model.to_bytes())
}

/// Shared state for the finance fetch endpoint.
struct FinanceState {
    provider: YahooFinanceProvider,
    cache: Mutex<Cache>,
}

/// Execute a batch of `Task::FinanceFetch` operations and return results.
///
/// The frontend calls `Model::take_tasks()` after `evaluate()`, sends the
/// tasks here, feeds the results back via `Model::complete_task()`, and
/// re-evaluates to display actual values.
#[post("/api/finance/fetch", format = "json", data = "<tasks>")]
async fn finance_fetch(
    state: &State<FinanceState>,
    tasks: Json<Vec<FinanceFetchTask>>,
) -> Json<Vec<Result<f64, FinanceError>>> {
    let mut results = Vec::new();

    for task in tasks.iter() {
        {
            let cache = state.cache.lock().unwrap();
            if let Some(cached) = cache.get(&task.ticker, &task.attribute) {
                results.push(cached.clone());
                continue;
            }
        }

        let result = state.provider.fetch(&task.ticker, &task.attribute).await;

        state
            .cache
            .lock()
            .unwrap()
            .insert(&task.ticker, &task.attribute, result.clone());

        results.push(result);
    }

    Json(results)
}

#[launch]
fn rocket() -> _ {
    let finance_state = FinanceState {
        provider: YahooFinanceProvider::new(),
        cache: Mutex::new(Cache::new()),
    };

    let mut rocket = rocket::build()
        .attach(IronCalcDB::init())
        .manage(finance_state)
        .mount(
            "/",
            routes![
                upload,
                download,
                share,
                get_model,
                get_model_list,
                finance_fetch
            ],
        );

    if let Ok(frontend_path) = std::env::var("IRONCALC_WEBAPP_DIR") {
        if !frontend_path.is_empty() {
            rocket = rocket.mount("/", rocket::fs::FileServer::from(frontend_path));
        }
    }

    rocket
}
