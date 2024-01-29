# Android build

Android needs a debug keystore file to install to devices and distribute non-release APKs. Android Studio would have generated on and stored it in `~/.android` on Linux and macOS / `C:\Users\<user>\.android\` on Windows.

In case it's not already generated, run:

```bash
keytool -keyalg RSA -genkeypair -alias androiddebugkey -keypass android -keystore debug.keystore -storepass android -dname "CN=Android Debug,O=Android,C=US" -validity 9999 -deststoretype pkcs12
```

This generates a keystore with keystore user `androiddebugkey` and keystore password `android`.

## Linking Gradle to JDK

Godot does not respect `$JAVA_HOME`, so we have to manually link JDK before building Android .

```
sudo ln -sfn /opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk /Library/Java/JavaVirtualMachines/openjdk-17.jdk
```

## Configuring Google Play Games Services

We rely on the [`godot-play-game-services`](https://github.com/Iakobs/godot-play-game-services) Android plugin for Google Play Games integration, with tweaks to the source code to better integrate with our codebase.

To enable the plugin, [set up Google Play Games Services](https://developers.google.com/games/services/console/enabling). The rough steps to follow on the guide are:

1. create a Play Games Services project
2. create OAuth consent screen in Google Cloud Platform
3. create credentials
4. add the Play Games Services SDK to your APK to use the APIs
5. add testers to your project

When the application ID is generated, fill it in on the "Godot Play Games Services" dock menu.

After that, we can build the debug / release version of the application.

### Testing Google Play Games Services

For Play Games Services to work on a debug build (using a debug keystore for signing), we need to include the emails being used for testing as Play Games Services testers.

Go to [Google Play Console](https://play.google.com/console/u/0/developers) → Play Games Services → Setup and management → Testers and add any emails to be used for testing.
