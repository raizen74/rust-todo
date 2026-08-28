# Rust nanoservices

Full stack to-do application with a modular design implementing Rust workspaces containing an authenticated to-do application. It is organized as two nanoservices, each split into domain logic, data access, and HTTP networking layers. The same auth contract can be used either by compiling auth into the to-do process or by calling a separately deployed auth service over HTTP.

## Workspace Structure

```text
.
|-- Cargo.toml                         # Root Cargo workspace
|-- glue/                              # Shared errors, tokens, and integrations
|   |-- Cargo.toml
|   `-- src/
|       |-- errors.rs                  # Shared service errors and Actix responses
|       |-- token.rs                   # JWT HeaderToken and Actix extractor
|       `-- lib.rs
|-- nanoservices/
|   |-- auth/
|   |   |-- core/                      # Authentication and user use cases
|   |   |-- dal/                       # User schemas, persistence, migrations
|   |   |-- kernel/                    # Auth boundary: in-process or HTTP
|   |   `-- networking/actix_server/   # Standalone auth HTTP server
|   |-- user-session-cache/
|   |   |-- cache-module/              # Rust Redis dynamic library
|   |   `-- cache-client/              # Redis client used by application crates
|   `-- to-do/
|       |-- core/                      # To-do use cases
|       |-- dal/                       # To-do schemas and storage adapters
|       `-- networking/actix_server/  # To-do HTTP server
|-- ingress/                           # Combined app and embedded frontend
|   |-- Cargo.toml
|   |-- index.html                     # Frontend shell embedded at compile time
|   |-- tasks.json                     # Ingress task/configuration data
|   |-- scripts/
|   |   `-- run_server.sh               # Build frontend and run ingress
|   `-- src/
|       `-- main.rs                    # Compose views, migrations, and assets
|-- frontend/                          # TypeScript frontend
|   |-- package.json                   # Frontend dependencies and scripts
|   |-- package-lock.json
|   |-- esbuild.js                     # Frontend bundler configuration
|   |-- tsconfig.json
|   |-- serve.json
|   |-- public/                        # Generated assets embedded by ingress
|   |   |-- index.html
|   |   |-- bundle.js
|   |   `-- bundle.css
|   `-- src/
|       |-- index.tsx                  # Frontend entrypoint
|       |-- App.css
|       |-- Login.css
|       |-- api/                       # Auth and to-do HTTP clients
|       |-- components/                # Login, create-item, and item views
|       `-- interfaces/                # Frontend data types
|-- compose.yaml                       # Development PostgreSQL service
|-- scripts/
|   `-- basic_test.sh                  # Basic project test script
|-- commands.txt                       # Example curl and run commands
|-- output.txt                         # Project output notes
|-- tasks.json                         # Workspace task definitions
`-- rustfmt.toml                       # Rust formatting configuration
```

The root `Cargo.toml` is a Cargo workspace. Individual crates can be selected with `cargo run -p <package>` or `cargo build -p <package>`.

## Glue Workspace

The `glue` crate contains functionality shared across the nanoservices. It keeps cross-cutting concerns out of the auth and to-do implementations while allowing those crates to use the same types and conventions:

- `NanoServiceError` and `NanoServiceErrorStatus` provide a common error and status model.
- With the `actix` feature, `NanoServiceError` implements Actix Web's `ResponseError`, converting service failures into HTTP status codes and JSON error responses.
- `HeaderToken` defines the JWT claims used between clients, the to-do server, and the auth server. It encodes and decodes the user's `unique_id` using the shared `JWT_SECRET` environment variable. `HeaderToken` implements the actix `FromRequest` trait to decode and extract the `unique_id` automatically from the incoming request.
- The `safe_eject!` macro converts lower-level errors into `NanoServiceError` values with an appropriate status and optional context.

The nanoservice crates depend on these shared contracts rather than defining incompatible error or token handling for each service. The `glue` crate is also used by the auth kernel, so both its direct database path and its HTTP path return the same error and user-facing types.

## User Session Cache

The `nanoservices/user-session-cache` workspace adds Redis-backed sessions to the application. It contains two complementary crates:

### `cache-module`

`cache-module` is a Rust dynamic library compiled as a `cdylib`. Its Dockerfile builds `libcache_module.so`, copies it into a Redis image, and starts Redis with the library loaded through `--loadmodule`. The `cache` service in `compose.yaml` builds this image and exposes Redis on port `6379`.

Because the module runs inside Redis, its commands operate directly on Redis data structures through the Redis Modules API instead of implementing session logic as an external polling service. It registers these custom commands:

| Command | Behavior |
| --- | --- |
| `login.set <unique_id> <timeout_mins> <user_id>` | Creates or resets the `user_session_<unique_id>` hash, records the last interaction time, resets the request counter, and stores the permanent database user ID. |
| `update.set <unique_id>` | Checks that the session exists, expires it when its timeout is exceeded, increments the interaction counter, refreshes the last-interaction timestamp, and returns the stored permanent user ID when valid. |
| `logout.set <unique_id>` | Deletes the user's session hash. |

Each session is represented by a Redis hash keyed as `user_session_<unique_id>`. The hash stores `last_interacted`, `timeout_mins`, `counter`, and `perm_user_id`. A session returns `TIMEOUT` after its configured inactivity period and returns `REFRESH` after more than 20 interactions, allowing the application layer to renew it.

### `cache-client`

`cache-client` is the asynchronous Rust client used by the application workspaces to communicate with Redis. It connects using `CACHE_API_URL` and exposes Rust functions for `login`, `logout`, and `update`, translating Redis responses such as `NOT_FOUND`, `TIMEOUT`, and `REFRESH` into `NanoServiceError` or `UserSessionStatus` values.

The client is compiled into `auth-kernel`, which gives the auth and to-do networking layers a typed Rust boundary instead of requiring them to know Redis command syntax. The cache module owns the data-structure operations; the cache client owns the application-to-Redis connection and response mapping.

## Layered Architecture

Each nanoservice separates **core**, **DAL**, and **networking** responsibilities:

```text
HTTP request
		|
		v
Actix networking layer  ->  core use case  ->  DAL transaction trait
																									 |
																									 v
																		 concrete descriptor implementation
```

### Core

The core crates contain application use cases and domain-facing request/response types. They do not need to know whether persistence is PostgreSQL, a file, or another implementation. A core function is generic over the DAL traits required by that operation.

For example, to-do operations are expressed through traits such as `GetAll`, `SaveOne`, `DeleteOne`, and `UpdateOne`. The core receives a descriptor type implementing those traits and delegates persistence to it.

The auth core provides user operations such as creating a user, logging in, and retrieving a user by unique ID. It returns `TrimmedUser` for lookups so password hashes are not exposed to callers.

### Data access layer (DAL)

The DAL crates define schemas, database connections, migrations, and transaction traits. They also provide concrete descriptor types:

- `auth-dal` currently provides `SqlxPostGresDescriptor` for PostgreSQL-backed users.
- `to-do-dal` provides `SqlxPostGresDescriptor` for PostgreSQL and `JsonFileDescriptor` for JSON-file storage.

Each descriptor implements the relevant transaction traits. The core and networking layers therefore depend on capabilities rather than a storage implementation. To add another backend, define a descriptor and implement the existing traits for it. The use-case code can remain unchanged; the networking layer only needs to select that descriptor when registering its routes.

The to-do DAL exposes these storage features:

| Feature | Effect |
| --- | --- |
| `json-file` | Enables JSON-file persistence and `serde_json` |
| `sqlx-postgres` | Enables PostgreSQL persistence, migrations, and `dotenv` |

## PostgreSQL Tables and Migrations

The PostgreSQL implementation uses SQLx migrations stored next to each service's DAL. The migrations are versioned SQL files, and each DAL calls `sqlx::migrate!("./migrations")` through its `run_migrations` function. SQLx records applied migrations in its migration tracking table, so migrations are applied incrementally and are not recreated on every startup.

The auth DAL owns the auth schema:

| Table | Columns and purpose |
| --- | --- |
| `users` | `id` (`SERIAL` primary key), unique `email`, hashed `password`, and unique `unique_id`. |

The to-do DAL owns the to-do schema:

| Table | Columns and purpose |
| --- | --- |
| `to_do_items` | `id` (`SERIAL` primary key), required `title`, and required `status`. |
| `user_connections` | Composite primary key of `user_id` and `to_do_id`, associating users with their items. |

The initial to-do migration creates `to_do_items`; a later migration adds `user_connections`. This keeps schema changes append-only and makes future changes explicit, reviewable, and repeatable. `user_connections` is the ownership join table: each row associates a `users.id` with a `to_do_items.id`, and its composite primary key prevents the same user/item association from being recorded twice. The current migration defines the columns and primary key but does not declare SQL foreign-key constraints, so the relationship is maintained by the DAL transaction functions.

The PostgreSQL DAL exploits this association as follows:

- `SaveOne` inserts the item into `to_do_items`, then inserts `(user_id, to_do_id)` into `user_connections` using the newly created item's ID.
- `GetAll` selects from `to_do_items` only when the item ID appears in a subquery over `user_connections` for the supplied `user_id`.
- `DeleteOne` uses the supplied user ID when removing the matching row from `user_connections` after deleting the item.
- `UpdateOne` accepts a user ID at the trait boundary, but the current PostgreSQL SQL update filters only by item ID and does not yet check `user_connections`.

Consequently, the association currently provides user scoping for creation and retrieval, and maintains the ownership record during deletion. To guarantee that PostgreSQL updates and deletes can never affect another user's item, those item-selection queries must also join or subquery `user_connections` with the authenticated `user_id`; the JSON-file descriptor already incorporates the user ID into its lookup key.

Both the standalone auth server and the standalone to-do server run their own DAL migrations at startup. The combined `ingress` binary runs both migration sets before starting its HTTP server, so a fresh development database can be initialized by the application processes themselves.

### Actix-server networking

The networking crates adapt Actix Web requests to core use cases and choose concrete descriptors at the route boundary. The standalone auth server registers user creation, user lookup, and login routes. The to-do server registers get, create, delete, and update routes.

The current to-do Actix server selects `to_do_dal::...::SqlxPostGresDescriptor`, so its executable uses PostgreSQL. `JsonFileDescriptor` remains available as an alternative DAL implementation for consumers that wire routes to it.

## Auth Nanoservice

The auth nanoservice is split into:

1. `auth-core`: user and authentication use cases.
2. `auth-dal`: user models, password hashing, PostgreSQL transactions, and migrations.
3. `auth-networking/actix_server`: the standalone Actix HTTP API.

Passwords are hashed with Argon2 when a user is created. Login uses HTTP Basic credentials and returns a token containing the user identity. Auth routes are served under `/api/v1`:

### Auth core operations

The `auth-core` workspace provides three generic operations. Each operation receives a DAL descriptor whose implementation supplies the required database transaction:

| Operation | Behavior |
| --- | --- |
| `auth/login` | Retrieves a user by email, verifies the supplied password against the stored Argon2 hash, and returns an encoded `HeaderToken` containing the user's `unique_id`. |
| `users/create` | Builds a new user, generates its unique ID, hashes the password, and saves the user in the database. |
| `users/get` | Calls `get_by_unique_id` to retrieve a user and returns the password-free `TrimmedUser` representation. |

These operations are generic over the auth DAL traits `GetByEmail`, `SaveOne`, and `GetByUniqueId`. The standalone Actix server supplies `SqlxPostGresDescriptor` when it registers the routes below.

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/api/v1/users/create` | Create a user |
| `GET` | `/api/v1/users/get` | Return the user identified by the token |
| `GET` | `/api/v1/auth/login` | Authenticate and return a token |

The standalone auth server listens on `127.0.0.1:8081` and runs auth migrations at startup.

## To-do Nanoservice

The to-do nanoservice is split into:

1. `to-do-core`: to-do use cases.
2. `to-do-dal`: to-do models, user/item connections, migrations, and storage descriptors.
3. `to-do-networking/actix_server`: the authenticated Actix API.

The to-do server listens on `127.0.0.1:8001`. Its protected endpoints accept the token through the `token` header:

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/api/v1/get/all` | Get the current user's items |
| `POST` | `/api/v1/create` | Create an item for the current user |
| `DELETE` | `/api/v1/delete/{name}` | Delete one of the current user's items |
| `PATCH` | `/api/v1/update` | Update an item |

Every to-do handler follows the same authorization flow:

1. Extract the token and its `unique_id`.
2. Call the auth kernel's `GetUserSession` operation with that `unique_id`.
3. The kernel calls Redis `update.set` and obtains the cached permanent `users.id`.
4. Stop with an unauthorized error if the session is missing or expired.
5. Pass the authenticated user's database ID to the to-do core operation.

This keeps authentication and session validation at the networking boundary and ensures that all to-do operations receive a server-resolved user ID before the core logic runs. The normal request path uses the session cache rather than querying the PostgreSQL `users` table for every to-do request.

### User-scoped operations

The authenticated user's database `id` is the ownership boundary for to-do data. The networking handler obtains it only after the auth kernel has resolved the token's `unique_id` through the session cache; the handler then passes that `user.id` into the to-do core function. The core forwards the ID to the DAL trait implementation rather than accepting an arbitrary user ID from the request body or URL.

The DAL uses that ID together with the `user_connections` association when creating and retrieving PostgreSQL-backed items. `SaveOne` records the new `(user_id, to_do_id)` connection, and `GetAll` returns only items connected to the supplied user. The JSON-file descriptor applies the same ownership idea by including the user ID in its storage key.

The authenticated ID is passed through the core for update and delete as well, but the current PostgreSQL `UpdateOne` query filters only by item ID. Its `DeleteOne` query filters by title first and uses the user ID when removing the connection. Therefore, the association is not yet a complete PostgreSQL authorization guard for those two mutations; their item-selection queries need to include the authenticated user ID to prevent cross-user manipulation at the database level.

### To-do data model

The to-do DAL defines the data structures shared by the DAL, core, and networking layers:

| Struct | Role |
| --- | --- |
| `NewToDoItem` | Client input for creation. Contains a `title` and a `TaskStatus` value. |
| `ToDoItem` | Persisted item. Contains an integer `id`, `title`, and status string. With `sqlx-postgres`, it also derives `sqlx::FromRow`. |
| `AllToDoItems` | Presentation result containing separate `pending` and `done` vectors. |
| `UserConnection` | Association between an auth user ID and a to-do item ID. |
| `TaskStatus` | The creation-time status enum, currently `PENDING` or `DONE`. |

`AllToDoItems::from_vec` groups persisted items into the `pending` and `done` collections returned by the get operation. The user connection model allows the PostgreSQL implementation to query and maintain items belonging to a particular authenticated user.

### To-do functionality

The to-do core exposes four generic use cases:

- **Create:** saves a `NewToDoItem` for the authenticated user's numeric ID and returns the created `ToDoItem`.
- **Get all:** loads the user's items through `GetAll` and groups them into `AllToDoItems`.
- **Delete:** removes an item by title for the authenticated user through `DeleteOne`.
- **Update:** changes an existing `ToDoItem` for the authenticated user through `UpdateOne`.

The HTTP handlers compose these operations so create, delete, and update can return the user's refreshed item list where applicable. The DAL decides whether the operation is backed by PostgreSQL or JSON-file storage; the to-do core's use-case behavior and public data types remain the same.

## Auth Kernel

`auth-kernel` is the integration boundary between authentication and the to-do views. Its purpose is to turn the authenticated identity extracted from the JWT into the internal database user ID required by the to-do application to allow/deny operations on the items. The kernel exposes one stable function:

```rust
pub async fn get_user_by_unique_id(
		id: String,
) -> Result<TrimmedUser, NanoServiceError>
```

The function has two implementations selected through Cargo features:

Of the three operations provided by `auth-core`, the kernel exposes only the user lookup operation. It is not an auth facade for login or user creation: those operations are exposed by the standalone auth HTTP server and are handled by its auth networking layer. For normal protected requests, the to-do views first pass the JWT extractor's `unique_id` to the kernel's `GetUserSession` operation. That operation looks up the session in Redis and returns the cached permanent `users.id` to the to-do core and DAL. The kernel calls `get_user_by_unique_id` only when Redis reports that the session needs to be refreshed; that lookup can use the `users` table directly or the configured auth HTTP service, depending on the selected feature.

That lookup and ownership flow is:

```text
JWT unique_id
	-> auth-kernel
	-> users.unique_id
	-> users.id
	-> user_connections.user_id
	-> user_connections.to_do_id
	-> owned to-do items
```

The resulting `users.id` is passed by the to-do view into the to-do core functions. The to-do DAL uses that ID with `user_connections` to identify which item IDs belong to the authenticated user, allowing the CRUD layer to accept or reject operations according to ownership rather than trusting a user ID supplied by the client.

The kernel's direct path invokes `auth-core`'s `users::get::get_by_unique_id`; its HTTP path invokes the equivalent `/api/v1/users/get` endpoint. Both paths return the same `TrimmedUser`, including the internal `users.id` needed by the to-do views.

The kernel also compiles the `cache-client` workspace and exposes Redis session operations through the `RedisSessionDescriptor`. Its `LoginUserSession` trait delegates to the cache client's `login` function, while its `GetUserSession` trait delegates to the cache client's `update` function. This keeps Redis access behind the same descriptor-and-trait pattern used by the database layers.

The session traits expose these function signatures:

```rust
pub trait GetUserSession {
	fn get_user_session(
		unique_id: String,
	) -> impl Future<Output = Result<UserSession, NanoServiceError>>;
}

pub trait LoginUserSession {
	fn login_user_session(
		address: &str,
		user_id: &str,
		timeout_mins: usize,
		perm_user_id: i32,
	) -> impl Future<Output = Result<(), NanoServiceError>>;
}
```

`GetUserSession` accepts the JWT `unique_id` and returns a `UserSession` containing the permanent database user ID. `LoginUserSession` stores that association in Redis using the cache address, session identity, timeout in minutes, and permanent user ID.

The kernel's `UserSession` value is the application-level representation of the permanent user ID returned by Redis:

```rust
pub struct UserSession {
	pub user_id: i32,
}
```

The Redis-backed implementation builds this value by delegating session state management to the cache module:

```rust
pub async fn get_session_redis(
	unique_id: String,
) -> Result<UserSession, NanoServiceError> {
	let address = std::env::var("CACHE_API_URL").map_err(|error| {
		NanoServiceError::new(error.to_string(), NanoServiceErrorStatus::BadRequest)
	})?;
	let session_status = cache_client::update(&address, &unique_id).await?;

	match session_status {
		UserSessionStatus::Ok(user_id) => Ok(UserSession { user_id }),
		UserSessionStatus::Refresh => {
			let user = get_user_by_unique_id(unique_id.clone()).await?;
			cache_client::login(&address, &unique_id, 20, user.id).await?;
			Err(NanoServiceError::new(
				"Session refreshed; request must be retried".to_string(),
				NanoServiceErrorStatus::Unknown,
			))
		}
	}
}
```

The cache module's internal `UserSession` implementation derives the Redis key as `user_session_<unique_id>`, updates `last_interacted`, and stores `timeout_mins`, `counter`, and `perm_user_id` in the Redis hash. Its timeout check deletes expired keys, increments the interaction counter, and returns `OK`, `TIMEOUT`, or `REFRESH`. The kernel maps a successful `OK` response to the `UserSession { user_id }` value consumed by the to-do views.

The two server integrations use those kernel operations differently:

- The auth Actix server calls `LoginUserSession` after a successful `auth/login`. It stores the JWT `unique_id`, a 20-minute timeout, and the permanent `users.id` in Redis.
- The to-do Actix server calls `GetUserSession` in its get, create, update, and delete views. It passes the `unique_id` extracted by `HeaderToken` to Redis and receives the permanent `user_id` needed by the to-do core.

When a to-do request reaches the kernel, the key is looked up in Redis first: `cache-client::update` invokes Redis `update.set` with the JWT's `unique_id`. A valid session returns the cached permanent user ID, so the request does not need to query the PostgreSQL `users` table on every operation. `NOT_FOUND` and `TIMEOUT` responses become unauthorized errors. When the cache reports `REFRESH`, the kernel uses its configured direct or HTTP `get_user_by_unique_id` path to retrieve the permanent `users.id`, then calls the cache client's `login` function to reset the Redis session with a 20-minute timeout. The refresh branch currently returns an internal error after renewing the key because it does not issue a second `update.set`; the documented cache-first flow therefore reflects the implementation's current behavior rather than implying that the request continues successfully after refresh.

The request sequence is therefore:

```text
to-do view
	-> HeaderToken extracts unique_id
	-> auth-kernel GetUserSession
	-> cache-client update
	-> Redis update.set
		|-- valid: return cached users.id
		|-- missing/timeout: return unauthorized
		`-- refresh threshold: get_user_by_unique_id -> cache-client login -> Redis login.set
```

### Descriptor-backed user lookup

In the direct database deployment, the kernel does not contain SQL or access the database pool itself. Instead, it selects the auth DAL descriptor and passes it to the generic auth-core function:

```rust
get_by_unique_id_core::<SqlxPostGresDescriptor>(id).await?
```

The lookup proceeds through the layers as follows:

1. `auth-kernel` receives the `unique_id` from the authenticated to-do request and selects `SqlxPostGresDescriptor`.
2. `auth-core::api::users::get::get_by_unique_id` is generic over the `GetByUniqueId` trait, so it can work with any descriptor implementing that capability.
3. The `GetByUniqueId` implementation for `SqlxPostGresDescriptor` delegates to the auth DAL's PostgreSQL function.
4. The DAL executes `SELECT * FROM users WHERE unique_id = $1` using its shared SQLx PostgreSQL connection pool.
5. The resulting `User` is converted into `TrimmedUser`, which omits the password hash, and the result is returned through the kernel.

If no matching row exists, the DAL returns a `NotFound` `NanoServiceError`. Database failures are converted into the same shared error type, allowing the networking layer to produce the corresponding HTTP response without coupling the to-do server to SQLx details.

### `core-postgres`

With `core-postgres`, the kernel enables `auth-core` and calls the auth core directly using `SqlxPostGresDescriptor`. The auth core then uses the auth DAL's PostgreSQL implementation in the same process.

```text
to-do HTTP handler
		-> auth-kernel::get_user_by_unique_id
		-> auth-core::get_by_unique_id<SqlxPostGresDescriptor>
		-> auth-dal PostgreSQL transaction
```

This is the default mode of `to-do-actix-server` through its `auth-core-postgres` feature.

### `http`

With `http`, the kernel enables `reqwest` and `dotenv`, reads `AUTH_API_URL`, and sends a request to:

```text
{AUTH_API_URL}/api/v1/users/get
```

It forwards the user identity in the `token` header, calls the independently deployed auth server, and deserializes its `TrimmedUser` response.

```text
to-do HTTP handler
		-> auth-kernel::get_user_by_unique_id
		-> HTTP GET to AUTH_API_URL
		-> standalone auth Actix server
		-> auth-core -> auth-dal PostgreSQL transaction
```

The kernel preserves the same function signature in both modes. The to-do core and its handlers do not need to know whether auth is local or remote.

In `http` mode, the kernel's own process does not use an auth database descriptor. It creates a `HeaderToken` from the supplied identity and calls the auth server's `/api/v1/users/get` endpoint. The standalone auth server then follows the descriptor-backed path above: its Actix handler invokes auth core with `SqlxPostGresDescriptor`, and that descriptor queries the `users` table. Thus the database lookup remains owned by the auth DAL in either deployment; only the process boundary changes.

This is the **key deployment flexibility of the project**: `auth-kernel` gives the to-do server **two interchangeable auth deployments without changing any lines in the to-do server's handlers, core calls, or authorization flow**. Only the Cargo feature selected for the build changes:

| To-do server command feature | Enables | Behavior |
| --- | --- | --- |
| `auth-core-postgres` | `auth-kernel/core-postgres` | Compile the auth core and its PostgreSQL DAL into the to-do server and perform the lookup directly |
| `auth-http` | `auth-kernel/http` | compile the HTTP client path and call an independently deployed auth microservice over HTTP |

The server's default feature is `auth-core-postgres`. The two modes are selected at compile time, so the same networking code can be deployed with either an embedded auth implementation or a service boundary.

The session-cache integration is shared by both Actix servers: `auth-actix-server` compiles `auth-kernel` to write sessions during login, and `to-do-actix-server` compiles the same kernel to read/update sessions during protected requests. The cache-client dependency and `RedisSessionDescriptor` keep these integrations consistent. Ingress composes both servers into one binary, so it also includes the kernel-based Redis session flow for its auth and to-do views.

In both cases the to-do server calls the same `get_user_by_unique_id` API, receives the same `TrimmedUser`, and passes the same `user.id` into the to-do core. The feature boundary replaces the auth implementation behind that API; it does not create a second to-do application or require an alternate set of endpoint code.

## Ingress Workspace

The `ingress` workspace is the deployable modular monolith for the application. It composes the nanoservices into one Rust binary while preserving their internal core, DAL, and networking boundaries.

Ingress injects the views of each nanoservice into one Actix application:

```rust
App::new()
	.configure(auth_views_factory)
	.configure(to_do_views_factory)
```

This means the auth and to-do routes can be assembled into the same server without copying their endpoint implementations. Ingress also runs the auth and to-do migration functions at startup, so the binary owns the complete backend composition.

### Shared kernel feature requirement

Both Actix server crates depend on `auth-kernel`. When they are compiled as separate executables, each deployment can select the kernel implementation appropriate to that process. When both servers are compiled into the same `ingress` binary, they must be compiled with the **same `auth-kernel` feature**.

This requirement prevents Cargo feature and dependency conflicts in the shared ingress dependency graph. The auth server's `auth-kernel` dependency is currently configured with `core-postgres`, and the to-do server's default `auth-core-postgres` feature forwards that same feature to the kernel. Consequently, the current ingress build uses the direct PostgreSQL-backed kernel path for both servers.

The matching configuration is:

```text
auth-actix-server  -> auth-kernel/core-postgres
to-do-actix-server -> auth-core-postgres -> auth-kernel/core-postgres
ingress            -> composes both servers with one kernel feature set
```

The `auth-http` option remains available when the to-do server is deployed independently and the auth server runs as a separate process. It should not be mixed with the auth server's `core-postgres` kernel configuration when both Actix servers are linked into ingress; changing the ingress deployment mode requires aligning the kernel feature configuration of both server crates first.

The frontend is built into `frontend/public`, and `rust-embed` **embeds those generated assets into the Rust binary at compile time**. Ingress serves the embedded `index.html` and static assets directly through Actix, while API requests are routed to the injected nanoservice views. A catch-all route supports frontend navigation and avoids treating API paths as frontend resources.

As a result, one `ingress` binary can serve the complete application:

```text
single Rust process
	|-- embedded frontend assets
	|-- auth Actix views
	|-- to-do Actix views
	|-- auth migrations
	`-- to-do migrations
```

This is a **modular monolith** rather than a separate rewrite of the services. The same nanoservice views used by independently deployed servers are injected into ingress, allowing the deployment shape to change from independently deployed auth/to-do microservices to a **single frontend-plus-backend Rust binary** without changing the core use cases.

## Running Locally

### Prerequisites

- Rust and Cargo
- Docker and Docker Compose
- Node.js and npm for the frontend

Start PostgreSQL and the Redis session cache:

```sh
docker compose up -d db cache
```

The development database in `compose.yaml` uses:

```text
host: localhost
port: 5432
database: to_do
user: username
password: mysecretpassword
```

The Redis session cache uses:

```text
host: localhost
port: 6379
url: redis://127.0.0.1:6379
```

Set the database URL for commands that use PostgreSQL:

```sh
export TO_DO_DB_URL='postgresql://username:mysecretpassword@localhost:5432/to_do'
export CACHE_API_URL='redis://127.0.0.1:6379'
```

### Combined application via ingress workspace

`ingress` runs both auth and to-do APIs in one process + embeds and serves the compiled frontend. Build the frontend first, then run:

```sh
cd frontend
npm install
npm run build
cd ..
cargo run -p ingress
```

The combined application listens on `0.0.0.0:8001` and runs both auth and to-do migrations.

### Independently deployed auth and to-do servers

In this deployment shape, the frontend is built and served separately from the two Rust API processes. The frontend build produces static files in `frontend/public`; it does not need a Rust server to be hosted.

Build the frontend from the repository root:

```sh
cd frontend
npm install
npm run build
```

For local development, serve the generated frontend in one terminal:

```sh
npm run serve
```

This serves `frontend/public` on the default `http://localhost:3000`. The frontend detects that development origin and sends API requests to the to-do server at `http://localhost:8001`. For deployment, publish the contents of `frontend/public` with any static web server or CDN. Configure the frontend's API base URL for the public to-do server origin when the frontend is not running on the local development port.

Run the independently deployed auth server in another terminal:

```sh
CACHE_API_URL='redis://127.0.0.1:6379' \
cargo run -p auth-actix-server
```

Run the to-do server with HTTP auth in a third terminal:

```sh
AUTH_API_URL='http://127.0.0.1:8081' \
CACHE_API_URL='redis://127.0.0.1:6379' \
cargo run -p to-do-actix-server --no-default-features --features auth-http
```

For embedded auth, run only the to-do server with its default feature:

```sh
CACHE_API_URL='redis://127.0.0.1:6379' \
cargo run -p to-do-actix-server
```

Both PostgreSQL-backed modes require `TO_DO_DB_URL` to be set in the environment used by the process. Auth login and to-do session updates also require `CACHE_API_URL` to point to the Redis container.

The independently deployed arrangement therefore consists of three deployable pieces: the static frontend, the to-do Actix server and the auth Actix server.

## Example Requests

Create a user on the standalone auth server:

```sh
curl -X POST http://127.0.0.1:8081/api/v1/users/create \
	-H 'Content-Type: application/json' \
	-d '{"email":"test@gmail.com","password":"password"}'
```

Log in to receive a token:

```sh
curl -u test@gmail.com:password \
	-X GET http://127.0.0.1:8081/api/v1/auth/login
```

Get the authenticated user by unique ID:

```sh
curl -X GET http://127.0.0.1:8081/api/v1/users/get \
	-H 'token: <JWT>'
```

The user lookup endpoint does not take the ID as a path parameter. The auth server decodes the user's `unique_id` from the JWT in the `token` header and returns the matching `TrimmedUser`.

Create a to-do item for that user:

```sh
curl -X POST http://127.0.0.1:8001/api/v1/create \
	-H 'Content-Type: application/json' \
	-H 'token: <JWT>' \
	-d '{"title":"code","status":"PENDING"}'
```

Get all to-do items belonging to the authenticated user:

```sh
curl -X GET http://127.0.0.1:8001/api/v1/get/all \
	-H 'token: <JWT>'
```

Update a to-do item by sending its persisted ID, title, and status:

```sh
curl -X PATCH http://127.0.0.1:8001/api/v1/update \
	-H 'Content-Type: application/json' \
	-H 'token: <JWT>' \
	-d '{"id":2,"title":"code review","status":"DONE"}'
```

Delete a to-do item by its title:

```sh
curl -X DELETE http://127.0.0.1:8001/api/v1/delete/code-review \
	-H 'token: <JWT>'
```

When `auth-http` is enabled, the token is received by the to-do server and the kernel performs the user lookup against `AUTH_API_URL`. When auth is embedded, the same request is resolved through the compiled auth core and DAL instead.