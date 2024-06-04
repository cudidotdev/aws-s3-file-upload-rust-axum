#![allow(unused_must_use)]
#![allow(dead_code)]

use aws_sdk_s3::primitives::ByteStream;
use axum::{
  body::Bytes,
  extract::{DefaultBodyLimit, Multipart, State},
  http::{header, StatusCode},
  response::{IntoResponse, Response},
  routing::post,
  Router,
};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::limit::RequestBodyLimitLayer;

#[tokio::main]
async fn main() {
  dotenvy::dotenv();

  let config = aws_config::load_from_env().await;
  let s3_client = aws_sdk_s3::Client::new(&config);

  let app = Router::new()
    .route("/upload", post(upload_hander))
    .layer(DefaultBodyLimit::disable())
    .layer(RequestBodyLimitLayer::new(10 * 1024 * 1023))
    .with_state(s3_client);

  let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

  axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct File {
  key: String,
  successful: bool,
  url: String,
  file_name: String,
  content_type: String,
  #[serde(skip_serializing)]
  bytes: Bytes,
}

async fn upload_hander(
  State(s3_client): State<aws_sdk_s3::Client>,
  mut multipart: Multipart,
) -> Result<Response, Response> {
  let mut files = vec![];
  let bucket_name = std::env::var("AWS_BUCKET_NAME").unwrap_or_default();

  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
  {
    if let Some("files") = field.name() {
      let file_name = field.file_name().unwrap_or_default().to_owned();
      let content_type = field.file_name().unwrap_or_default().to_owned();
      let key = uuid::Uuid::new_v4().to_string();
      let url = format!("https://{bucket_name}.s3.amazonaws.com/{key}");

      let bytes = field
        .bytes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

      files.push(File {
        file_name,
        content_type,
        bytes,
        key,
        url,
        successful: false,
      })
    }
  }

  for file in &mut files {
    let body = ByteStream::from(file.bytes.to_vec());

    let res = s3_client
      .put_object()
      .bucket(&bucket_name)
      .content_type(&file.content_type)
      .content_length(file.bytes.len() as i64)
      .key(&file.key)
      .body(body)
      .send()
      .await;

    file.successful = res.is_ok();
  }

  Ok(
    (
      StatusCode::OK,
      [(header::CONTENT_TYPE, "application/json")],
      serde_json::json!(files).to_string(),
    )
      .into_response(),
  )
}
