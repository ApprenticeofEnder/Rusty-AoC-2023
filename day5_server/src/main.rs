use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    path::PathBuf,
};

use actix_files::NamedFile;
use actix_web::{
    error, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use argparse::{ArgumentParser, Store};
use base64::{
    engine::{general_purpose, GeneralPurpose},
    Engine as _,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use zip::write::FileOptions;

const MAX_SIZE: usize = 262_144;
const B64_SCHEME: GeneralPurpose = general_purpose::STANDARD;
//---STRUCTS---

#[derive(Serialize, Deserialize)]
struct FileData {
    filename: String,
    b64_data: String,
}

struct Config {
    filename: String,
}

//---HELPERS---

fn read_target_contents(filepath: &PathBuf) -> std::io::Result<Vec<u8>> {
    let mut file: File = File::open(filepath)?;
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

fn make_zip(filepath_str: &String, zip_path: &PathBuf) -> std::io::Result<()> {
    let filepath: PathBuf = PathBuf::from(filepath_str);

    let zip: File = File::create(zip_path)?;
    let mut zip_writer: zip::ZipWriter<File> = zip::ZipWriter::new(zip);
    let options: FileOptions =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let filename: std::ffi::OsString = filepath.file_name().unwrap().to_os_string();
    let contents: Vec<u8> = read_target_contents(&filepath)?;

    zip_writer
        .start_file(filename.to_str().unwrap(), options)
        .unwrap();
    zip_writer.write(&contents)?;

    zip_writer.finish()?;
    Ok(())
}

//---ROUTES---

#[get("/")]
async fn file_dl(data: web::Data<Config>, req: HttpRequest) -> HttpResponse {
    let mut zip_path: PathBuf = env::current_dir().unwrap();
    zip_path.push("req.zip");
    let zip_path: PathBuf = zip_path;
    make_zip(&data.filename, &zip_path).unwrap();

    let zip_file = File::open(&zip_path).unwrap();

    let named_zip: NamedFile = NamedFile::from_file(zip_file, &zip_path).unwrap();

    let res: HttpResponse = named_zip.into_response(&req);

    fs::remove_file(&zip_path).unwrap();

    res
}

#[post("/fix")]
async fn file_fix(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let file_data_src: FileData = serde_json::from_slice::<FileData>(&body)?;

    let mut file_data: Vec<u8> = B64_SCHEME.decode(&file_data_src.b64_data).unwrap();

    let fix_bytes: [u8; 2] = [0x41, 0x43];

    fix_bytes.iter().enumerate().for_each(|(index, byte)| {
        file_data[index] = *byte;
    });

    let patched_data_b64 = B64_SCHEME.encode(file_data);

    Ok(HttpResponse::Ok().json(FileData {
        b64_data: patched_data_b64,
        filename: file_data_src.filename,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut filename: String = Default::default();

    {
        let mut ap: ArgumentParser<'_> = ArgumentParser::new();
        ap.set_description("Rusty AoC 2023 Day 5: DOS RevEng (SERVER)");
        ap.refer(&mut filename)
            .add_argument("file", Store, "The file to zip and serve")
            .required();
        ap.parse_args_or_exit();
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Config {
                filename: filename.clone(),
            }))
            .service(file_fix)
            .wrap(middleware::Compress::default())
            .service(file_dl)
    })
    .bind(("0.0.0.0", 8080))?;

    let addresses: Vec<std::net::SocketAddr> = server.addrs();

    addresses.iter().for_each(|addr| {
        println!("Listening on {:?}", addr);
    });

    server.run().await
}
