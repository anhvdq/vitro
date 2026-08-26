#!/bin/sh
set -eu

set -a
. ./.env
set +a

schema="${PG_DATABASE_SCHEMA:-public}"
export DATABASE_URL="postgresql://${PG_DATABASE_HOST}:${PG_DATABASE_PORT}/${PG_DATABASE_DB}?user=${PG_DATABASE_USERNAME}&password=${PG_DATABASE_PASSWORD}&options=--search_path=${schema}"

sqlx migrate run
