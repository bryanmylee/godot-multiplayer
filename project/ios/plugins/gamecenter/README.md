# Godot iOS GameCenter plugin

## Setup

In addition to setting up Game Center in Godot, Game Center has to be [configured in App Store Connect](https://developer.apple.com/documentation/gamekit/enabling_and_configuring_game_center/).

To do so, create the app on the [Apps](https://appstoreconnect.apple.com/apps) page and **enable the Game Center capability**.

## Architecture

The GameCenter plugin revolves around an event polling system. Events are sent to the iOS plugin via method calls, and events are received from the plugin via `get_pending_event_count` and `pop_pending_event`.

## Methods

### Authorization

`authenticate()` - Performs user authentication. Generates a new `authentication` event.

```json
{
  "type": "authentication",
  "result": "ok",
  "alias": "eatingdumplings",
  "displayName": "eatingdumplings",
  "player_id": "T:_48001ab8bf56c9fdcdc242bac5db3b0d"
}
```

`is_authenticated()` - Returns authentication state.

### GameCenter methods

`post_score(Dictionary score_dictionary)` - Reports a score data to iOS `GameCenter`. Generates new event with `post_score` type.  
`award_achievement(Dictionary achievent_dictionary)` - Reports progress of achievement data to iOS `GameCenter`. Generates new event with `award_achievement` type.  
`reset_achievements()` - Resets all achievement progress for the local player. Generates new event with `reset_achievements` type.  
`request_achievements()` - Loads previously submitted achievement progress for the local player from iOS `GameCenter`. Generates new event with `achievements` type.  
`request_achievement_descriptions()` - Downloads the achievement descriptions from iOS `GameCenter`. Generates new event with `achievement_descriptions` type.  
`show_game_center(Dictionary screen_dictionary)` - Displays Game Center information of your game. Generates new event with `show_game_center` type when information screen closes.  
`request_identity_verification_signature()` - Creates a signature for a third-party server to authenticate the local player. Generates new event with `identity_verification_signature` type.

## Properties

## Events reporting

`get_pending_event_count()` - Returns number of events pending from plugin to be processed.  
`pop_pending_event()` - Returns first unprocessed plugin event.
