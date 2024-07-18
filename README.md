# Sukab Restaurant

The name Sukab is taken from the fictional character made by [famous Indonesian Poet, Seno G. Ajidarma](https://en.wikipedia.org/wiki/Seno_Gumira_Ajidarma).

# How to Run the Server

1. Spin up a PostgreSQL server, a minimum version of PostgreSQL 14.
2. Import the schema and data from the `./src/db/schema/sukab-restaurant.sql` file.
   This file will create: 1) Database `sukab_restaurant`, 2) Table `orders`, 3) Table `menus`.
   The file also contains the seed values for `menus` table.
3. Build the app, run `cargo build`
4. Set these environment variables:
    ```
    export PG_HOST=localhost
    export PG_USER=<your_user>
    export PG_PWD=<your_pwd>
    ```
5. Run the app by executing this in the terminal: `./target/...`

# Appendix 1: Environment Variables

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
