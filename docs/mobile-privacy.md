# Papr Android data and privacy notes

These notes describe the current Android app behavior for internal RC review.

| Data / action | Where it goes |
| --- | --- |
| Subscriptions, folders, articles, reading state, tags, rules, highlights, settings and completed AI results | Stored in the app's local SQLite database. Article HTML and images may also be retained in app-private cache. |
| Feed discovery | The curated directory search uses a list bundled with the app. Adding or refreshing a feed, extracting full text and loading remote media contact the relevant site. |
| AI summary, summary-local follow-up and translation | On user action, article text or HTML blocks and the selected prompt go to the AI endpoint configured in the active profile. A connection test sends a short test prompt. The selected provider processes these requests under its own terms. |
| FreshRSS / Miniflux | When connected, the configured server receives synchronization requests for subscriptions, folders and supported article read/star state. The app does not upload local article bodies, cached media, AI results or API keys in these requests. |
| AI and sync credentials | Encrypted ciphertext is stored in app-private preferences; its AES key is held in Android Keystore. The database stores references and connection metadata, not the secret. The credential preference file is excluded from Android cloud backup and device transfer. |

Android may back up other app-private data under the current manifest rules; the credential preference file is explicitly excluded. A restore onto another device can therefore require re-entering AI or sync credentials. Users can disconnect a sync account or delete an AI profile in Settings. **Reset preferences** keeps content and credentials. **Clear all data** requests Android to erase the app's local database, cache, preferences and Keystore entries and closes the app; it does not erase content already exported through the system document picker or data held by a feed, AI, or sync provider. Uninstalling the app also removes its local app data under Android's normal behavior.

Static package check on 2026-09-28: the internal `0.1.0+2` APK declares Internet, network state, wake lock, vibration, boot receiver, notification and media/short foreground-service permissions. The manifest points to backup rules that exclude `papr_ai_credentials.xml` from cloud backup and device transfer; the credential store uses the matching `papr_ai_credentials` preference name. These checks inspect the package and source rules; restore behavior has not been exercised on a device.

AI deletion check on 2026-09-28: a temporary profile with a placeholder secret was created in the API 37 `Medium_Phone` emulator. Its app-private credential preference file had one entry before in-app deletion and zero afterward; the profile list returned to empty. This did not inspect the Android Keystore alias directly or test sync credential deletion.

App-data-clear check on 2026-09-28: a second temporary AI profile and Japanese language setting were present before the in-app confirmation. Android ended the process and the app-private preferences directory was absent. A cold relaunch succeeded, language returned to English, the AI profile list was empty, and the credential preference file was absent. The database file, Keystore alias and sync credential behavior were not directly inspected.

Before distributing the internal RC, confirm actual backup behavior on supported OS versions, third-party endpoints used by the configured providers, and the internal APK's included SDKs.
