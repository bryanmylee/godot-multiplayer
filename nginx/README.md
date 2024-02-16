# Server Setup

Assume a Linux server with `{ip}` allocated.

## Domain

For a top-level domain e.g. `game.com`, add an A Record with `@` as the host and `{ip}` as the value.

For a subdomain e.g. `{sub}.game.com`, add an A record with `{sub}` as the host and `{ip}` as the value.

Assume the domain name `{domain}` is properly configured for the server.

## TLS with Let's Encrypt and NGINX

We use an NGINX proxy to provide TLS by forwarding our connections. We use the Let's Encrypt client Certbot to manage our SSL certificates.

### Install the Let's Encrypt client

```bash
apt-get update
sudo apt-get install -y certbot python3-certbot-nginx
```

### Configure NGINX

Certbot automatically configures NGINX for SSL/TLS by modifying the `server` block in the configuration that contains a `server_name` directive using the domain name we request a certificate for.

To start, create an NGINX configuration with the templates in [`nginx`](nginx). Replace `{domain}` in the examples.

> Note that we can have multiple `server {...}` blocks as long as their `listen` configurations do not overlap. This allows us to run multiple services on one server machine. _However, for Certbot configuration, it's best to set up one block first_.

Enable the site by creating a symlink of the config in `/etc/nginx/sites-enabled/`.

```bash
sudo ln -s /etc/nginx/sites-available/{domain} /etc/nginx/sites-enabled/{domain}
```

To disable the site, simply delete the symlink.

### Configure SSL/TLS with Certbot

The NGINX plugin for Certbot automatically reconfigures and reloads the NGINX configuration to enable SSL.

Make sure to also include the following server block to serve files for the ACME challenge.

```nginx
server {
        listen 80 default_server;
        listen [::]:80 default_server;
        root /var/www/html;
        server_name {domain};
}
```

Then, run `certbot`.

```bash
sudo certbot --nginx -d {domain}
```

> Make sure that port `80` is not already being listened to and/or redirected. Check that `/etc/nginx/sites-enabled/default` is removed.

With Certbot v1.21.0, a schedule will already be created to automatically renew the certificate.

After the certificate is issued, update the NGINX configuraton to make sure the reconfiguration is correct.

For the game server, we expect the final configuration to be similar to:

```nginx
server {
        # Forward WebSocket connection requests.
        listen [::]:9000-9249 ssl ipv6only=on;
        listen 9000-9249 ssl;
        server_name {domain};

        location / {
                proxy_set_header        Host $host;
                proxy_set_header        X-Real-IP $remote_addr;
                proxy_set_header        X-Forwarded-For $proxy_add_x_forwarded_for;
                proxy_set_header        X-Forwarded-Proto $scheme;

                # Fix the "It appears that your reverse proxy set up is broken" error.
                proxy_pass              http://127.0.0.1:1$server_port;
                # Prevent dropped WebSocket connections.
                proxy_read_timeout      1d;

                # Forward the WebSocket upgrade request.
                proxy_http_version      1.1;
                proxy_set_header        Upgrade $http_upgrade;
                proxy_set_header        Connection "upgrade";
        }

        ssl_certificate /etc/letsencrypt/live/{domain}/fullchain.pem; # managed by Certbot
        ssl_certificate_key /etc/letsencrypt/live/{domain}/privkey.pem; # managed by Certbot
        include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
        ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot
}

server {
        if ($host = {domain}) {
                return 301 https://$host$request_uri;
        } # managed by Certbot

        listen [::]:80;
        listen 80;
        server_name {domain};
        return 404; # managed by Certbot
}
```

> If there are multiple server blocks for multiple services on the same domain name, each block should have the same `ssl_certificate`, `ssl_certificate_key`, `include`, and `ssl_dhparam` fields created by Certbot, and each `listen` field should have the `ssl` option enabled.

Check the configuration one last time, then reload NGINX.

```bash
nginx -t && nginx -s reload
```

## Firewall

Once the services are behind NGINX's TLS proxy, make sure to expose the ports on the firewall.

```bash
sudo ufw allow 443/tcp
sudo ufw allow 9000:9249/tcp
```

To check the firewall status, use `ufw status`.
