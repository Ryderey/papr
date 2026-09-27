# Papr Android data and privacy notes

These notes describe the current Android app behavior for RC review. They are a product data-flow summary, not a completed store privacy declaration.

| Data / action | Where it goes |
| --- | --- |
| Subscriptions, folders, articles, reading state, tags, rules, highlights, settings and completed AI results | Stored in the app's local SQLite database. Article HTML and images may also be retained in app-private cache. |
| Feed discovery | The curated directory search uses a list bundled with the app. Adding or refreshing a feed, extracting full text and loading remote media contact the relevant site. |
| AI summary, summary-local follow-up and translation | On user action, article text or HTML blocks and the selected prompt go to the AI endpoint configured in the active profile. A connection test sends a short test prompt. The selected provider processes these requests under its own terms. |
| FreshRSS / Miniflux | When connected, the configured server receives synchronization requests for subscriptions, folders and supported article read/star state. The app does not upload local article bodies, cached media, AI results or API keys in these requests. |
| AI and sync credentials | Encrypted ciphertext is stored in app-private preferences; its AES key is held in Android Keystore. The database stores references and connection metadata, not the secret. The credential preference file is excluded from Android cloud backup and device transfer. |

Android may back up other app-private data under the current manifest rules; the credential preference file is explicitly excluded. A restore onto another device can therefore require re-entering AI or sync credentials. Users can disconnect a sync account or delete an AI profile in Settings. **Reset preferences** keeps content and credentials. **Clear all data** requests Android to erase the app's local database, cache, preferences and Keystore entries and closes the app; it does not erase content already exported through the system document picker or data held by a feed, AI, or sync provider. Uninstalling the app also removes its local app data under Android's normal behavior.

Before publishing a store privacy declaration, confirm actual backup behavior on supported OS versions, third-party endpoints used by the configured providers, and the final release build's included SDKs.
