# Server Build

## Building the image

Make sure to have Docker installed.

To build the game server image, run the command below from the repository root:

```bash
docker build --platform linux/x86_64 --tag {custom_tag} -f build/server/Dockerfile export/server
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

Certain parts of the application are controlled via environment variables. Pass them into the container with `-e` / `--env`.

```bash
docker run --detach --name {name} --publish 19000:9000 \
	--env SERVER_TIMEOUT=10.0 \
	--env SERVER_ID=5432 \
	{custom_tag}
```