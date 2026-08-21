pub mod create;
pub mod delete;
pub mod get;
pub mod update;
use actix_web::web::{ServiceConfig, get, scope};

pub fn basic_actions_factory(app: &mut ServiceConfig) {
    app.service(scope("/api/v1").route("get/all", get().to(get::get_all)));
}
