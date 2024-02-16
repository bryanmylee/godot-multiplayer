# Web Client Build

## Building the image

Make sure to have Docker installed.

To build the web client image, run the command below from the repository root:

```bash
docker build --platform linux/x86_64 --tag {custom_tag} -f build/web/Dockerfile export/web
```

## Pushing images to Docker Hub

To push the image, run:

```bash
docker image push {custom_tag}
```

> To push the image onto the Docker Hub Container Image Library, `{custom_tag}` has to be prefixed by your Docker Hub username e.g. `bryanmylee/server`.

## Pulling images on servers

Run a container with the latest version of the image and it will be pulled automatically.

## Running containers with images

The web client should run on port `10443`, which will be forwarded to `443` by NGINX. Refer to the [Web Client NGINX configuration](.././../nginx/templates/web_client.nginx).

Certain parts of the application are controlled via environment variables. Pass them into the container with `-e` / `--env`.

```bash
docker run --detach --name {name} --publish 10443:9000 {custom_tag}
```
