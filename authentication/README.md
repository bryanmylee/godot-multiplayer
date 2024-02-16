# Authentication Server

## Architecture

The authentication server allows users to sign in with multiple types of providers, including OAuth 2.0, Steam, Google Play Games, and Apple Game Center.

Each provider's actions are placed under a route with their name i.e. `/auth/oauth2/`, `/auth/steam/`, `/auth/play-games/` and `/auth/game-center/`.

### Signing in

The authentication server uses access tokens to grant authority to user resources.

Upon launch, the client attempts to sign in with its provider identity at `/sign-in/`.

If `(provider_type, provider_id)` already exists, simply update the provider information and return the access and refresh token.

If the provider is newly seen, check for any matching providers based on _email_ if possible. If a matching provider is found, return an unconfirmed state to the client, allowing the client to choose whether it wants to link the current provider to the existing account or create a new account.

The user can either confirm a link with `/link-account/` which will add the provider under the existing user, or explicitly create a new account with `/create-account/`. Both routes will return a valid access and refresh token.

## Database setup

We use `diesel-cli` for database migrations.

To setup `diesel`, run:

```bash
diesel setup --database-url='postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:15432/${POSTGRES_DB}'
```

Take the variables from the project `compose.yaml` file.

Refer to [`diesel-cli`](https://crates.io/crates/diesel_cli) for usage documentation.

## Authentication Providers

### Google Play Games Services

Setting up Google Play Games on the server requires a dedicated [OAuth 2.0 Client ID](https://console.cloud.google.com/apis/credentials), separate from the Android client or the Web client. The server's client type should be "Web application".

### Steam

Using Steam's Web API requires a [Steamworks Web API publisher authentication key](https://partner.steamgames.com/doc/webapi_overview/auth).

To create a Publisher Web API key:

1. As a user with administrative rights in your Steamworks account, first visit your groups list by going to Users & Permissions, then Manage Groups.
2. From the list of groups, select or create a group that contains the App IDs for which you wish to have access with the WebAPI key.
3. Then click into that group to view the users and applications in that group.
4. If you have administrative permissions, you should then see the option to "Create WebAPI Key" on the right-hand side. Or you should see the key listed if it has already been created.
