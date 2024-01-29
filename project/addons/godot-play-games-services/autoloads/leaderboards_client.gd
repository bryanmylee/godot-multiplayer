# class_name PlayGamesLeaderboardsClient
extends Node
## Client with  leaderboards functionality.
##
## This autoload exposes methods and signals to show the game leaderboards, as well
## as submitting and retrieving the player's score.

## Signal emitted after calling the [method submit_score] method.[br]
## [br]
## [param is_submitted]: Indicates if the score was submitted or not.[br]
## [param leaderboard_id]: The leaderboard id.
signal score_submitted(is_submitted: bool, leaderboard_id: String)

## Signal emitted after calling the [method load_player_score] method.[br]
## [br]
## [param leaderboard_id]: The leaderboard id.[br]
## [param score]: The score of the player. It can be null if there is an error
## retrieving it.
signal score_loaded(leaderboard_id: String, score: PlayGamesLeaderboardScore)

## Signal emitted after calling the [method load_all_leaderboards] method.[br]
## [br]
## [param leaderboards]: An array containing all the leaderboards for the game.
## The array will be empty if there was an error loading the leaderboards.
signal all_leaderboards_loaded(leaderboards: Array[PlayGamesLeaderboard])

## Signal emitted after calling the [method load_leaderboard] method.[br]
## [br]
## [param leaderboard]: The leaderboard. It can be null if there is an error
## retrieving it.
signal leaderboard_loaded(leaderboard: PlayGamesLeaderboard)

## Time span for leaderboards.
enum PlayGamesLeaderboardTimeSpan {
	TIME_SPAN_DAILY = 0, ## A leaderboard that resets everyday.
	TIME_SPAN_WEEKLY = 1, ## A leaderboard that resets every week.
	TIME_SPAN_ALL_TIME = 2 ## A leaderboard that never resets.
}

## Collection type for leaderboards.
enum PlayGamesLeaderboardCollection {
	COLLECTION_PUBLIC = 0, ## A public leaderboard.
	COLLECTION_FRIENDS = 3 ## A leaderboard only with friends.
}

## Score order for leadeboards.
enum PlayGamesLeaderboardScoreOrder {
	SCORE_ORDER_LARGER_IS_BETTER = 1, ## Scores are sorted in descending order.
	SCORE_ORDER_SMALLER_IS_BETTER = 0 ## Scores are sorted in ascending order.
}

func _ready() -> void:
	_connect_signals()

func _connect_signals() -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.scoreSubmitted.connect(func(is_submitted: bool, leaderboard_id: String):
			score_submitted.emit(is_submitted, leaderboard_id)
		)
		GodotPlayGamesServices.android_plugin.scoreLoaded.connect(func(leaderboard_id: String, json_data: String):
			var safe_dictionary := GodotPlayGamesServices.json_marshaller.safe_parse_dictionary(json_data)
			score_loaded.emit(leaderboard_id, PlayGamesLeaderboardScore.new(safe_dictionary))
		)
		GodotPlayGamesServices.android_plugin.allLeaderboardsLoaded.connect(func(leaderboards_json: String):
			var safe_array := GodotPlayGamesServices.json_marshaller.safe_parse_array(leaderboards_json)
			var leaderboards: Array[PlayGamesLeaderboard] = []
			for dictionary: Dictionary in safe_array:
				leaderboards.append(PlayGamesLeaderboard.new(dictionary))
			all_leaderboards_loaded.emit(leaderboards)
		)
		GodotPlayGamesServices.android_plugin.leaderboardLoaded.connect(func(json_data: String):
			var safe_dictionary := GodotPlayGamesServices.json_marshaller.safe_parse_dictionary(json_data)
			leaderboard_loaded.emit(PlayGamesLeaderboard.new(safe_dictionary))
		)

## Use this method to show all leaderbords for this game in a new screen.
func show_all_leaderboards() -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.showAllLeaderboards()

## Use this method to show a specific leaderboard in a new screen.[br]
## [br]
## [param leaderboard_id]: The leaderboard id.
func show_leaderboard(leaderboard_id: String) -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.showLeaderboard(leaderboard_id)

## Use this method to show a specific leaderboard for a given time span in a new screen.[br]
## [br]
## [param leaderboard_id]: The leaderboard id.[br]
## [param time_span]: The time span for the leaderboard. See the [enum PlayGamesLeaderboardTimeSpan] enum.
func show_leaderboard_for_time_span(leaderboard_id: String, time_span: PlayGamesLeaderboardTimeSpan) -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.showLeaderboardForTimeSpan(leaderboard_id, time_span)

## Use this method to show a specific leaderboard for a given time span and 
## collection type in a new screen.[br]
## [br]
## [param leaderboard_id]: The leaderboard id.[br]
## [param time_span]: The time span for the leaderboard. See the [enum PlayGamesLeaderboardTimeSpan] enum.[br]
## [param collection]: The collection type for the leaderboard. See the [enum PlayGamesLeaderboardCollection] enum.
func show_leaderboard_for_time_span_and_collection(
	leaderboard_id: String,
	time_span: PlayGamesLeaderboardTimeSpan,
	collection: PlayGamesLeaderboardCollection
) -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.showLeaderboardForTimeSpanAndCollection(leaderboard_id, time_span, collection)

## Submits the score to the leaderboard for the currently signed in player. The score 
## is ignored if it is worse (as defined by the leaderboard configuration) than a previously
## submitted score for the same player.[br]
## [br]
## The method emits the [signal score_submitted] signal.[br]
## [br]
## [param leaderboard_id]: The leaderboard id.[br]
## [param score]: The raw score value.
func submit_score(leaderboard_id: String, score: int) -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.submitScore(leaderboard_id, score)

## Use this method and subscribe to the emitted signal to receive the score of the
## currently signed in player for the specified leaderboard, time span, and collection.[br]
## [br]
## The method emits the [signal score_loaded] signal.[br]
## [br]
## [param leaderboard_id]: The leaderboard id.[br]
## [param time_span]: The time span for the leaderboard. See the [enum PlayGamesLeaderboardTimeSpan] enum.[br]
## [param collection]: The collection type for the leaderboard. See the [enum PlayGamesLeaderboardCollection] enum.
func load_player_score(
	leaderboard_id: String,
	time_span: PlayGamesLeaderboardTimeSpan,
	collection: PlayGamesLeaderboardCollection
) -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.loadPlayerScore(leaderboard_id, time_span, collection)

## Use this method and subscribe to the emitted signal to receive the list of the game
## leaderboards.[br]
## [br]
## The method emits the [signal all_leaderboards_loaded] signal.[br]
## [br]
## [param force_reload]: If true, this call will clear any locally cached 
## data and attempt to fetch the latest data from the server. Send it set to [code]true[/code]
## the first time, and [code]false[/code] in subsequent calls, or when you want
## to clear the cache.
func load_all_leaderboards(force_reload: bool) -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.loadAllLeaderboards(force_reload)

## Use this method and subscribe to the emitted signal to receive a leaderboard.[br]
## [br]
## The method emits the [signal leaderboard_loaded] signal.[br]
## [br]
## [param leaderboard_id]: The leaderboard id.[br]
## [param force_reload]: If true, this call will clear any locally cached 
## data and attempt to fetch the latest data from the server. Send it set to [code]true[/code]
## the first time, and [code]false[/code] in subsequent calls, or when you want
## to clear the cache.
func load_leaderboard(leaderboard_id: String, force_reload: bool) -> void:
	if GodotPlayGamesServices.android_plugin:
		GodotPlayGamesServices.android_plugin.loadLeaderboard(leaderboard_id, force_reload)

## The score of a player for a specific leaderboard.
class PlayGamesLeaderboardScore:
	var display_rank: String ## Formatted string for the rank of the player.
	var display_score: String ## Formatted string for the score of the player.
	var rank: int ## Rank of the player.
	var raw_score: int ## Raw score of the player.
	var score_holder: PlayGamesPlayersClient.PlayGamesPlayer ## The player object who holds the score.
	var score_holder_display_name: String ## Formatted string for the name of the player.
	var score_holder_hi_res_image_uri: String ## Hi-res image of the player.
	var score_holder_icon_image_uri: String ## Icon image of the player.
	var score_tag: String ## Optional score tag associated with this score.
	var timestamp_millis: int ## Timestamp (in milliseconds from epoch) at which this score was achieved. 
	
	## Constructor that creates a Score from a [Dictionary] containing the properties.
	func _init(dictionary: Dictionary) -> void:
		if dictionary.has("displayRank"): display_rank = dictionary.displayRank 
		if dictionary.has("displayScore"): display_score = dictionary.displayScore
		if dictionary.has("rank"): rank = dictionary.rank
		if dictionary.has("rawScore"): raw_score = dictionary.rawScore
		if dictionary.has("scoreHolder"): score_holder = PlayGamesPlayersClient.PlayGamesPlayer.new(dictionary.scoreHolder)
		if dictionary.has("scoreHolderDisplayName"): score_holder_display_name = dictionary.scoreHolderDisplayName
		if dictionary.has("scoreHolderHiResImageUri"): score_holder_hi_res_image_uri = dictionary.scoreHolderHiResImageUri
		if dictionary.has("scoreHolderIconImageUri"): score_holder_icon_image_uri = dictionary.scoreHolderIconImageUri
		if dictionary.has("scoreTag"): score_tag = dictionary.scoreTag
		if dictionary.has("timestampMillis"): timestamp_millis = dictionary.timestampMillis
	
	func _to_string() -> String:
		var result := PackedStringArray()
		
		result.append("display_rank: %s" % display_rank)
		result.append("display_score: %s" % display_score)
		result.append("rank: %s" % rank)
		result.append("raw_score: %s" % raw_score)
		result.append("score_holder: {%s}" % str(score_holder))
		result.append("score_holder_display_name: %s" % score_holder_display_name)
		result.append("score_holder_hi_res_image_uri: %s" % score_holder_hi_res_image_uri)
		result.append("score_holder_icon_image_uri: %s" % score_holder_icon_image_uri)
		result.append("score_tag: %s" % score_tag)
		result.append("timestamp_millis: %s" % timestamp_millis)
		
		return ", ".join(result)

## A leaderboard.
class PlayGamesLeaderboard:
	var leaderboard_id: String ## The leaderboard id.
	var display_name: String ## The display name of the leaderboard.
	var icon_image_uri: String ## The URI to the leaderboard icon image.
	var score_order: PlayGamesLeaderboardScoreOrder ## The sorting order of the leaderboard, based on the score.
	## A list of variants of this leaderboard, based on the combination of the
	## leaderboard [enum PlayGamesLeaderboardTimeSpan] aPlayGamesCollectionollection].
	var variants: Array[PlayGamesLeaderboardVariant] = []
	
	## Constructor that creates a PlayGamesLeaderboard from a [Dictionary] containing the
	## properties.
	func _init(dictionary: Dictionary) -> void:
		if dictionary.has("leaderboardId"): leaderboard_id = dictionary.leaderboardId
		if dictionary.has("displayName"): display_name = dictionary.displayName
		if dictionary.has("iconImageUri"): icon_image_uri = dictionary.iconImageUri
		if dictionary.has("scoreOrder"): score_order = PlayGamesLeaderboardScoreOrder.get(dictionary.scoreOrder)
		if dictionary.has("variants"):
			for variant: Dictionary in dictionary.variants:
				variants.append(PlayGamesLeaderboardVariant.new(variant))
	
	func _to_string() -> String:
		var result := PackedStringArray()
		
		result.append("leaderboard_id: %s" % leaderboard_id)
		result.append("display_name: %s" % display_name)
		result.append("icon_image_uri: %s" % icon_image_uri)
		result.append("score_order: %s" % PlayGamesLeaderboardScoreOrder.find_key(score_order))
		
		for variant: PlayGamesLeaderboardVariant in variants:
			result.append("{%s}" % str(variant))
		
		return ", ".join(result)

## A specific variant of [enum PlayGamesLeaderboardTimeSpan] aPlayGamesCollectionollection] for a leaderboard.
class PlayGamesLeaderboardVariant:
	var display_player_rank: String ## The formatted rank of the player for this variant.
	var display_player_score: String ## The formatted score of the player for this variant.
	var num_scores: int ## The total number of scores for this variant.
	var player_rank: int ## The rank of the player for this variant.
	var player_score_tag: String ## The score tag of the player for this variant.
	var raw_player_score: int ## The score of the player for this variant.
	var has_player_info: bool ## Whether or not this variant contains score information for the player.
	var collection: PlayGamesLeaderboardCollection ## The type of [enum PlayGamesLeaderboardCollection] of this variant.
	var time_span: PlayGamesLeaderboardTimeSpan ## The type of [enum PlayGamesLeaderboardTimeSpan] of this variant.
	
	## Constructor that creates a PlayGamesLeaderboardVariant from a [Dictionary] containting
	## the properties.
	func _init(dictionary: Dictionary) -> void:
		if dictionary.has("displayPlayerRank"): display_player_rank = dictionary.displayPlayerRank
		if dictionary.has("displayPlayerScore"): display_player_score = dictionary.displayPlayerScore
		if dictionary.has("numScores"): num_scores = dictionary.numScores
		if dictionary.has("playerRank"): player_rank = dictionary.playerRank
		if dictionary.has("playerScoreTag"): player_score_tag = dictionary.playerScoreTag
		if dictionary.has("rawPlayerScore"): raw_player_score = dictionary.rawPlayerScore
		if dictionary.has("hasPlayerInfo"): has_player_info = dictionary.hasPlayerInfo
		if dictionary.has("collection"): collection = PlayGamesLeaderboardCollection.get(dictionary.collection)
		if dictionary.has("timeSpan"): time_span = PlayGamesLeaderboardTimeSpan.get(dictionary.timeSpan)
	
	func _to_string() -> String:
		var result := PackedStringArray()
		
		result.append("display_player_rank: %s" % display_player_rank)
		result.append("display_player_score: %s" % display_player_score)
		result.append("num_scores: %s" % num_scores)
		result.append("player_rank: %s" % player_rank)
		result.append("player_score_tag: %s" % player_score_tag)
		result.append("raw_player_score: %s" % raw_player_score)
		result.append("has_player_info: %s" % has_player_info)
		result.append("collection: %s" % str(collection))
		result.append("time_span: %s" % str(time_span))
		
		return ", ".join(result)
