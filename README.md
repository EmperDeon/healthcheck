# Healthcheck helper for applications and common dependencies

Supported:
- Timestamp from file
- RabbitMQ (Successful connection)
- PostgreSQL (Successful connection to DB and `SELECT 1`)
- Redis (Successful connection and `INFO server`)
- HTTP (Successful connection and 200 response)

Configuration is done mainly from command args, ENV and `.env` in current dir.

Command reference is printed by `healthchecks --help`.
