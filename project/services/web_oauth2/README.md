# OAuth2.0 in Godot

We use Google's OAuth2.0 client for our application, but this can be augmented in the future to support more OAuth2.0 providers.

## Setting up OAuth2.0 client

Go to the [Credentials page](https://console.cloud.google.com/apis/credentials) on Google Cloud Console and create a new OAuth client for the application.

> The Google Cloud Project should already be created for Google Play Games Service.

Head to the "OAuth consent screen" page and make sure to include any emails to be used for testing.

## OAuth2.0 Flow

Upon initialization, `WebOAuth2Service` requests an authorization code from Google's OAuth2.0 server on `https://accounts.google.com/o/oauth2/auth`.

The authorization code is returned on the redirect URI as the query parameter `code`. We set up our application to check for `code` in the query parameters and load it as such.

### Testing OAuth2.0 locally

To allow the default web debugging client on `http://localhost:8060/tmp_js_export.html` to receive the authorization code, we need to add its URI to the authorized redirect URIs on the Google Cloud console, under the "Credentials" page.
