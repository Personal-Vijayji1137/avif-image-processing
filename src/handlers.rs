use crate::utils::{convert_to_avif, generate_unique_id};
use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use aws_sdk_s3::Client as S3Client;
use futures_util::TryStreamExt as _;

#[get("/")]
pub async fn landing_page() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("api.html"))
}

#[post("/upload")]
pub async fn upload_handler(
    mut payload: Multipart,
    s3: web::Data<S3Client>,
) -> Result<HttpResponse, Error> {
    let mut file_bytes = Vec::new();
    while let Some(field) = payload.try_next().await? {
        let mut field = field;
        while let Some(chunk) = field.try_next().await? {
            file_bytes.extend_from_slice(&chunk);
        }
    }
    let avif_bytes = convert_to_avif(file_bytes, None).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("AVIF conversion failed: {}", e))
    })?;
    let key = format!("static/{}.avif", generate_unique_id());
    let bucket =
        std::env::var("AWS_IMAGE_BUCKET_NAME").expect("AWS_IMAGE_BUCKET_NAME env required");
    s3.put_object()
        .bucket(bucket)
        .key(key.clone())
        .body(avif_bytes.clone().into())
        .send()
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("s3 put object failed: {}", e))
        })?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "uploaded and converted to avif",
        "key": key
    })))
}

#[get("/{key:.*}")]
pub async fn image_handler(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    s3: web::Data<S3Client>,
) -> Result<HttpResponse, Error> {
    let key = path.into_inner();

    let bucket =
        std::env::var("AWS_IMAGE_BUCKET_NAME").expect("AWS_IMAGE_BUCKET_NAME env required");

    let resp = s3
        .get_object()
        .bucket(bucket)
        .key(&key)
        .send()
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("s3 get failed: {}", e)))?;

    let collected = resp.body.collect().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("failed collecting body: {}", e))
    })?;
    let data = collected.into_bytes();

    let width = query.get("w").and_then(|s| s.parse::<u32>().ok());
    let height = query.get("h").and_then(|s| s.parse::<u32>().ok());

    if width.is_none() && height.is_none() {
        Ok(HttpResponse::Ok().content_type("image/avif").body(data))
    } else {
        let vf = match (width, height) {
            (Some(w), Some(h)) => {
                format!(
                    "scale='if(gt(a,{w}/{h}),-1,{w})':'if(gt(a,{w}/{h}),{h},-1)',\
             crop={w}:{h}"
                )
            }
            (Some(w), None) => format!("scale={}: -1", w),
            (None, Some(h)) => format!("scale=-1:{}", h),
            _ => "scale=iw:ih".into(),
        };

        let out = convert_to_avif(data.to_vec(), Some(vf))
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

        Ok(HttpResponse::Ok().content_type("image/avif").body(out))
    }
}
