use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use axum::{
    extract::Multipart,
    routing::post,
};

use image::ImageFormat;
use image::ImageReader;
use std::io::Cursor;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/blur",post(handler_blur));

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