# Authentication Server

An NGINX proxy provides TLS by forwarding ports defined below:

| Internal service port | External port with TLS | Protocol | Description             |
| --------------------- | ---------------------- | -------- | ----------------------- |
| `18000`               | `8000`                 | HTTP     | The authentication API. |
