# Sukab Restaurant

The name Sukab is taken from the fictional character made by [famous Indonesian Poet, Seno G. Ajidarma](https://en.wikipedia.org/wiki/Seno_Gumira_Ajidarma).

# System Design

## High level overview

There are four HTTP REST endpoints:

| Method | Path | Description |
|--------|------|-------------|
| POST   | `/table/{table_number}/order` | Create new Order. |
| GET    | `/table/{table_number}/order` | List all Orders on a Table. |
| GET    | `/table/{table_number}/order/{order_id}` | Describe an Order on a Table. |
| DELETE | `/table/{table_number}/order/{order_id}` | Delete an Order on a Table. |

# How to Run the Server

1. Spin up a PostgreSQL server, a minimum version of PostgreSQL 14 is required.
2. Import the schema and data from the `./src/db/schema/sukab-restaurant.sql` file.
   This file will create:
    - Database `sukab_restaurant`,
    - Table `orders`,
    - Table `menus`.
    - Seed data for `menus` table.
3. Build the app, run `cargo build --release`
4. Set these environment variables:
    ```
    export PG_HOST=localhost
    export PG_USER=<your_user>
    export PG_PWD=<your_pwd>
    ```
5. Run the app by executing this command in the terminal: `./target/release/server`.

# How to Run the Client

1. Make sure the Server is running, if not, follow the guide above.
2. Run the app by executing this command in the terminal: `./target/release/client`.

# Appendix 1: Environment Variables

## Server Env Vars

|      Key      | Description | Required | Default |
|---------------|-------------|----------|---------|
|`RUST_LOG`     | env_logger log level/verbosity.|No|`debug`|
|`HTTP_HOST`    | Host for the HTTP server|No|`localhost`|
|`HTTP_PORT`    | Port for the HTTP server|No|`8080`|
|`PG_HOST`      | PostgreSQL host address, defaults to `localhost`.|Yes|`localhost`|
|`PG_PORT`      | PostgreSQL port, defaults to `5432`.|Yes|`5432`|
|`PG_USER`      | PostgreSQL username, defaults to `postgres`.|Yes|`5432`|
|`PG_PWD`       | PostgreSQL password, defaults to empty string.|Yes|`<empty string>`|
|`PG_DBNAME`    | PostgreSQL database name, defaults to `sukab_restaurant`.|No|`sukab_restaurant`|
|`COOK_TIME_MIN`| Minimum bound to get randomized Cook Time.|No|`5`|
|`COOK_TIME_MIN`| Maximum bound to get randomized Cook Time.|No|`15`|

## Client Env Vars

|      Key      | Description | Required | Default |
|---------------|-------------|----------|---------|
|`RUST_LOG`     | env_logger log level/verbosity.|No|`debug`|
|`SERVER_BASE_URL`     | Base URL for the Server |No|`http://localhost:8080`|
|`CLIENT_THREAD_COUNT`| Controls how many threads to spawn to send requests.|No|`10`|
