use actix_web::web;

use crate::handlers::{image_handler, landing_page, upload_handler};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(landing_page)
            .service(upload_handler)
            .service(image_handler),
    );
}
