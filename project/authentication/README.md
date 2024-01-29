# Authentication Design

For our cross-platform game, we use different authentication strategies, including Steam, Apple Game Center, Google Play Games Services, and OAuth 2.0.

## Platform Identity Providers

### Steam

Steam is initialized with the [`GodotSteam`](https://github.com/CoaguCo-Industries/GodotSteam) extension by CoaguCo-Industries.

The service provides more than just authentication, and is managed by [`steam_service.gd`](../services/steam_service.gd).

### Apple Game Center

iOS users are authenticated with Apple Game Center by default.

We manage a custom fork of the official [`godot-ios-plugins`](https://github.com/bryanmylee/godot-ios-plugins) repo, built for Godot 4.2.

[`apple_game_center_service.gd`](../services/apple_game_center_service.gd) provides typed Promise-based wrappers around the Game Center plugin.

> In addition to setting up Game Center in Godot, Game Center has to be [configured in App Store Connect](https://developer.apple.com/documentation/gamekit/enabling_and_configuring_game_center/). Refer to the [Game Center plugin document](../ios/plugins/gamecenter/README.md) for details about setting up Game Center.

### Google Play Games Services

Android users are authenticated with Google Play Games Services by default.

We manage a custom fork of [`Iakobs`](https://github.com/Iakobs/godot-play-game-services)/[`godot-play-games-services`](https://github.com/bryanmylee/godot-play-games-services) to reduce global namespace conflicts.

> In addition to setting up Play Games Services in Godot, Play Games Services has to be configured on the [Google Play Console](https://play.google.com/console/u/0/developers). Refer to the [Android build document](../android/README.md).