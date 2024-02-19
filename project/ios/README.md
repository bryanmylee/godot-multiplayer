# iOS Build

## Linking project files directly in Xcode

After exporting an iOS project, drag the Godot project folder `./project` into the root of Xcode's file browser, selecting **Create folder references**.

Move `<project_name>.pck` to the trash.

Open `<project_name>/Supporting Files/<project_name>-Info`, and add a new string entry with key `godot_path` and value set to the project folder name `project`.

> Make sure the project folder doesn't have the same name as the project itself to avoid signing issues in Xcode.

## Building plugins for Godot 4.0

We've recursively cloned `godot-ios-plugins` into this directory and set the `godot` submodule to the commit matching our current engine version.

```bash
git clone --recursive https://github.com/godotengine/godot-ios-plugins.git plugins/godot-ios-plugins
cd plugins/godot-ios-plugins/godot
git fetch
git checkout <godot_engine_commit>
```

To build the plugin, run the commands below:

```bash
scons platform=ios target=template_debug
cd .. # plugins/godot-ios-plugins
scons target=release_debug arch=arm64 simulator=no plugin=<plugin> version=4.0
./scripts/generate_static_library.sh <plugin> release_debug 4.0
./scripts/generate_xcframework.sh <plugin> release_debug 4.0
```

## Adding plugins to the project

Then, rename `./bin/<plugin>.release_debug.xcframework` to `./bin/<plugin>.xcframework` and move the directory to the project's iOS plugin directory [`project/ios/plugins/`](../ios/plugins/).

```bash
mv ./bin/<plugin>.release_debug.xcframework ../../project/ios/plugins/<plugin>/<plugin>.xcframework
```
