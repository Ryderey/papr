# Android RC release procedure

The Android release variant reads `mobile/android/key.properties` and no longer uses the debug signing key. The file and `*.jks` / `*.keystore` are already ignored by `mobile/android/.gitignore`; keep the keystore outside the repository and supply its path locally or through CI secrets. Do not commit passwords or the key file.

`key.properties` requires `storeFile`, `storePassword`, `keyAlias` and `keyPassword`, as described in the [Flutter Android release guide](https://docs.flutter.dev/deployment/android). On Windows, escape backslashes in a Java properties path or use forward slashes. Use the registered upload key for an existing Play app. A direct APK upgrade also requires the same app-signing certificate as the installed Alpha. Google Play requires the uploaded AAB to be signed with an upload key, then Play App Signing handles delivered APKs ([Android signing guide](https://developer.android.com/studio/publish/app-signing)).

1. Confirm package ID, `mobile/pubspec.yaml` version code/name, Alpha schema fixture and the signing certificate of the installed Alpha. Version code is currently `1`; it must increase for an upgrade candidate.
2. Put the protected upload key and local `key.properties` in place. Build with `cd mobile; flutter build appbundle --release` and, for direct device testing, `flutter build apk --release`.
3. Record the artifact SHA-256 and verify its signer with Android build tools. Install the release APK over an Alpha APK signed by the **same certificate**; retain the app data and check schema migration, launch and the subscription-to-reading flow. A debug-signed Alpha cannot be directly upgraded with a different release certificate.
4. Test a fresh install and clear-data semantics separately. Retain the AAB, signing certificate fingerprint, build version and device results in the P7 task record; never record passwords or private key bytes.

The previous 93.2 MB AAB was signed with the debug key and is only a build-gate artifact. Release signing, Alpha upgrade and Play internal-test readiness are **not yet verified**.
