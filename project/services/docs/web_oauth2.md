# Web OAuth 2.0 in Godot

We use Google's OAuth2.0 client for our application, but this can be augmented in the future to support more OAuth2.0 providers.

## Setting up the OAuth2.0 client

Go to the [Credentials page](https://console.cloud.google.com/apis/credentials) on Google Cloud Console and create a new OAuth client for the application.

> The Google Cloud Project should already be created for Google Play Games Service.

Head to the "OAuth consent screen" page and make sure to include any emails to be used for testing.

## OAuth2.0 Flow

Upon initialization, `WebOAuth2Service` requests an access code from Google's OAuth2.0 server by navigating to `https://accounts.google.com/o/oauth2/v2/auth`, which will redirect back to the web client on success with the access token in the URL hash under `access_token`.

Whenever the web client is initialized, it checks the URL fragment hash for the access code and consumes it if it exists by storing the access token into local storage along with an expiry date then removing the hash from the URL.

Whenever user information is needed, a request is made with the access token to Google's user info request URL.

## Testing OAuth2.0 locally

To allow the default web debugging client on `http://localhost:8060/tmp_js_export.html` to receive the authorization code, we need to add its URL to the authorized redirect URLs on the Google Cloud console, under the "Credentials" page.
