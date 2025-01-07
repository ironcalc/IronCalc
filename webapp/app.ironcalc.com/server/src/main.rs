#[macro_use]
extern crate rocket;

mod database;
mod id;

use std::io::{self, BufWriter, Cursor, Write};

use database::{add_model, get_model_list_from_db, select_model, IronCalcDB};
use ironcalc::base::Model as IModel;
use ironcalc::export::save_xlsx_to_writer;
use ironcalc::import::load_from_xlsx_bytes;
use rocket::data::{Data, ToByteUnit};
use rocket::http::{ContentType, Header};
use rocket::response::Responder;

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

    let bytes = data
        .open(MAX_SIZE_MB.megabytes())
        .into_bytes()
        .await
        .unwrap();
    if !bytes.is_complete() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "The file was not fully uploaded",
        ));
    };

    let model = IModel::from_bytes(&bytes).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Error creating model, '{e}'"))
    })?;

    let mut buffer: Vec<u8> = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = BufWriter::new(cursor);
        save_xlsx_to_writer(&model, &mut writer).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Error saving model: '{e}'"))
        })?;
        writer.flush().unwrap();
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "file was not fully uploaded",
        ));
    }
    add_model(db, &hash, &bytes).await?;
    println!("done share: '{}'", hash);
    Ok(hash)
}

#[get("/api/model/<hash>")]
async fn get_model(db: Connection<IronCalcDB>, hash: &str) -> io::Result<Vec<u8>> {
    let bytes = select_model(db, hash).await.unwrap();
    println!("Select model: '{}'", hash);
    Ok(bytes)
}

#[get("/api/list")]
async fn get_model_list(db: Connection<IronCalcDB>) -> io::Result<String> {
    let model_list = get_model_list_from_db(db).await.unwrap();
    println!("Model list: '{:?}'", model_list);
    Ok(model_list.join(","))
}

#[post("/api/upload/<name>", data = "<data>")]
async fn upload(data: Data<'_>, name: &str) -> io::Result<Vec<u8>> {
    println!("start upload");
    let bytes = data.open(MAX_SIZE_MB.megabytes()).into_bytes().await?;
    if !bytes.is_complete() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "file was not fully uploaded",
        ));
    }
    let workbook = load_from_xlsx_bytes(&bytes, name.trim_end_matches(".xlsx"), "en", "UTC")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Error loading model: '{e}'")))?;
    let model = IModel::from_workbook(workbook).map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Error creating model: '{e}'"))
    })?;
    println!("end upload");
    Ok(model.to_bytes())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(IronCalcDB::init())
        .mount("/", routes![upload, download, share, get_model, get_model_list])
}
