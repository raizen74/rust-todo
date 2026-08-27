pub mod create;
pub mod get;
use actix_web::web::{get, post, scope, ServiceConfig};
use auth_dal::users::descriptors::SqlxPostGresDescriptor;

pub fn users_factory(app: &mut ServiceConfig) {
    app.service(
        scope("/api/v1/users")
            .route(
                "create",
                post().to(create::create::<SqlxPostGresDescriptor>),
            )
            .route(
                "get",
                get().to(get::get_by_unique_id::<SqlxPostGresDescriptor>),
            ),
    );
}
