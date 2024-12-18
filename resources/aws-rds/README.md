# Shuttle AWS RDS

This plugin provisions databases on AWS RDS using [shuttle](https://www.shuttle.dev). The following three engines are supported:

- Postgres
- MySql
- MariaDB

## Usage

Add `shuttle-aws-rds` to the dependencies for your service.
Every engine is behind the following feature flags and attribute paths.
The default is to use native TLS.
You can also add `-rustls` after the feature flag, for example `postgres-rustls`.

| Engine   | Feature flag | Attribute path            |
|----------|--------------|---------------------------|
| Postgres | postgres     | shuttle_aws_rds::Postgres |
| MySql    | mysql        | shuttle_aws_rds::MySql    |
| MariaDB  | mariadb      | shuttle_aws_rds::MariaDB  |

An example using the Tide framework can be found on [GitHub](https://github.com/shuttle-hq/shuttle-examples/tree/main/tide/postgres)

### Options

Each engine can take in the following options:

| Option    | Type | Description                                                                             |
|-----------|------|-----------------------------------------------------------------------------------------|
| local_uri | &str | Don't spin up a local docker instance of the DB, but rather connect to this URI instead |
