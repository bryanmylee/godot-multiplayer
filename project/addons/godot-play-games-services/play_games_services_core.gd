class_name PlayGamesServicesCore
extends Node
## Main Autoload of the plugin, which contains a reference to the android plugin itself.
##
## This Autoload contains the entrypoint to the android code, but you don't need
## to use it directly. Some autoloads exposing the plugin functionality, as a wrapper
## for GDScript code, are also loaded with the plugin.[br]
## [br]
## This Autoload also calls the [code]initialize()[/code] method of the plugin,
## checking if the user is authenticated.

## Signal emitted after an image is downloaded and saved to the device.[br]
## [br]
## [param file_path]: The path to the stored file.
signal image_stored(file_path: String)

## Main entry point to the android plugin. With this object, you can call the 
## kotlin methods directly.
var android_plugin: Object

var achievements_client: PlayGamesAchievementsClient
var leaderboards_client: PlayGamesLeaderboardsClient
var players_client: PlayGamesPlayersClient
var sign_in_client: PlayGamesSignInClient
var snapshots_client: PlayGamesSnapshotsClient

## A helper JSON marshaller to safely access JSON data from the plugin.
var json_marshaller := JsonMarshaller.new()

func initialize() -> bool:
	var plugin_name := "godot-play-games-services"
	
	if not android_plugin:
		if not Engine.has_singleton(plugin_name):
			printerr("No plugin found!")
			return false
		
		print("Plugin found!")	
		android_plugin = Engine.get_singleton(plugin_name)
		android_plugin.initialize()
	
	android_plugin.imageStored.connect(func(file_path: String):
		image_stored.emit(file_path)
	)

	_initialize_clients()
	return true


func _initialize_clients() -> void:
	achievements_client = load("res://addons/godot-play-games-services/clients/achievements_client.gd").new(
		self
	)
	add_child(achievements_client)

	leaderboards_client = load("res://addons/godot-play-games-services/clients/leaderboards_client.gd").new(
		self
	)
	add_child(leaderboards_client)

	players_client = load("res://addons/godot-play-games-services/clients/players_client.gd").new(
		self
	)
	add_child(players_client)

	sign_in_client = load("res://addons/godot-play-games-services/clients/sign_in_client.gd").new(
		self
	)
	add_child(sign_in_client)

	snapshots_client = load("res://addons/godot-play-games-services/clients/snapshots_client.gd").new(
		self
	)
	add_child(snapshots_client)


## Displays the given image in the given texture rectangle.[br]
## [br]
## [param texture_rect]: The texture rectangle control to display the image.[br]
## [param file_path]: The file path of the image, for example user://image.png.
func display_image_in_texture_rect(texture_rect: TextureRect, file_path: String) -> void:
	if FileAccess.file_exists(file_path):
		var image := Image.load_from_file(file_path)
		texture_rect.texture = ImageTexture.create_from_image(image)
	else:
		print("File %s does not exist." % file_path)
