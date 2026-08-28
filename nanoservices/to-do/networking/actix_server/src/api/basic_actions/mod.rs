pub mod create;
pub mod delete;
pub mod get;
pub mod update;
use actix_web::web::{ServiceConfig, delete, get, patch, post, scope};
use auth_kernel::user_session::descriptors::RedisSessionDescriptor;
use to_do_dal::to_do_items::descriptors::SqlxPostGresDescriptor;

pub fn basic_actions_factory(app: &mut ServiceConfig) {
    app.service(
        scope("/api/v1")
            .route(
                "get/all",
                get().to(get::get_all::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
            )
            // .route("get/{name}", get().to(get::get_by_name::<SqlxPostGresDescriptor>))
            .route(
                "create",
                post().to(create::create::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
            )
            .route(
                "delete/{name}",
                delete()
                    .to(delete::delete_by_name::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
            )
            .route(
                "update",
                patch().to(update::update::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
            ),
    );
}
