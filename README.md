# An AWS S3 file upload service in Rust Axum

## Routes
```http
POST /upload
Content-Type: multipart/form-data
```

## Setup
Create a .env file and update it with your credentials

```env
AWS_ACCESS_KEY_ID=""
AWS_SECRET_ACCESS_KEY=""
AWS_REGION=""
AWS_BUCKET_NAME=""
```

Build and run 
```bash
cargo build && cargo run
```
