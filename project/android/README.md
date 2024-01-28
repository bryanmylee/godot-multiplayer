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
