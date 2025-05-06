use axum::extract::Query;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::{
    extract::Multipart,
    routing::{post,get},
};

use image::ImageFormat;
use image::ImageReader;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Cursor;
use exif::{Tag, In};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/jpeg",get(jpeg_orientation))
                                .route("/img", get(handler));


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler_blur(mut multipart: Multipart) -> impl IntoResponse {
    // let _ = mut multipart;

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // println!("Length of `{}` is {} bytes", name, data.len());

        let mut reader = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .expect("Cursor io never fails");
        // assert_eq!(reader.format(), Some(ImageFormat::Pnm));

        let image = reader.decode().unwrap();

        let blurred_img = image.blur(2.0);


        let mut buf = Cursor::new(Vec::new());
        blurred_img.write_to(&mut buf,ImageFormat::Jpeg).unwrap();

        return 
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/jpeg")],
                buf.into_inner()
            )
    }

    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/plain")],
        "No image is provided".into()
    )

}


#[derive(Debug, Deserialize)]
struct Params {
    path : String
}

#[derive(Serialize)]
struct ResponseOrientarion {
    orientation: Option<u8>,
    msg: String
}

async fn jpeg_orientation(Query(params) : Query<Params>) -> impl IntoResponse {


    println!("path from params: {}", params.path);
    let file = match File::open(&params.path) {
        Ok(file) => file,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ResponseOrientarion {
                    orientation: None,
                    msg: "Could not open the image , please provide the correct path : error {e}".to_owned()
                })
            );
        }
    };

    println!("0");

    // let file = std::fs::File::open()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).unwrap();

    println!("1");

    if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY)
                                    .and_then(|f| f.value.get_uint(0)) {
        (
            StatusCode::OK,
                Json(ResponseOrientarion {
                    orientation: Some(field as u8),
                    msg: "orientation tag found".to_string()
                })

        )
    }
    else{
        (
            StatusCode::BAD_REQUEST,
                Json(ResponseOrientarion {
                    orientation: None,
                    msg: "failed".to_string()
                })
        )
    }
    
}


async fn handler(Query(params) : Query<Params>) -> impl IntoResponse { 

    match fs::read(&params.path) {
        Ok(bytes) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/jpeg")],
                bytes
            )
        },
        Err(e) => {
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain")],
                "fILE not found".into()
            )
        }
    }
}