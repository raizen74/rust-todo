pub mod create;
pub mod delete;
pub mod get;
pub mod update;
use actix_web::web::{ServiceConfig, delete, get, patch, post, scope};
use auth_kernel::user_session::descriptors::RedisSessionDescriptor;
#[cfg(feature = "dal-json")]
use to_do_dal::to_do_items::descriptors::JsonFileDescriptor;
#[cfg(feature = "dal-postgres")]
use to_do_dal::to_do_items::descriptors::SqlxPostGresDescriptor;

pub fn basic_actions_factory(app: &mut ServiceConfig) {
    app.service(
        scope("/api/v1")
            .route(
                "get/all",
                #[cfg(feature = "dal-postgres")]
                get().to(get::get_all::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
                #[cfg(feature = "dal-json")]
                get().to(get::get_all::<JsonFileDescriptor, RedisSessionDescriptor>),
            )
            // .route("get/{name}", get().to(get::get_by_name::<SqlxPostGresDescriptor>))
            .route(
                "create",
                #[cfg(feature = "dal-postgres")]
                post().to(create::create::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
                #[cfg(feature = "dal-json")]
                post().to(create::create::<JsonFileDescriptor, RedisSessionDescriptor>),
            )
            .route(
                "delete/{name}",
                #[cfg(feature = "dal-postgres")]
                delete().to(delete::delete_by_name::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
                #[cfg(feature = "dal-json")]
                delete().to(delete::delete_by_name::<JsonFileDescriptor, RedisSessionDescriptor>),
            )
            .route(
                "update",
                #[cfg(feature = "dal-postgres")]
                patch().to(update::update::<SqlxPostGresDescriptor, RedisSessionDescriptor>),
                #[cfg(feature = "dal-json")]
                patch().to(update::update::<JsonFileDescriptor, RedisSessionDescriptor>),
            ),
    );
}
