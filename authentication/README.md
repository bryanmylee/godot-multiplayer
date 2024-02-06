# Authentication Server

An NGINX proxy provides TLS by forwarding ports defined below:

| Internal service port | External port with TLS | Protocol | Description             |
| --------------------- | ---------------------- | -------- | ----------------------- |
| `18000`               | `8000`                 | HTTP     | The authentication API. |

## Database setup

We use `diesel-cli` for database migrations.

To setup `diesel`, run:

```bash
diesel setup --database-url='postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:15432/${POSTGRES_DB}'
```

Take the variables from the project `compose.yaml` file.

Refer to [`diesel-cli`](https://crates.io/crates/diesel_cli) for usage documentation.
