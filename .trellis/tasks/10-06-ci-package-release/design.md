# Design

Use the approved design document as the design baseline. Keep `ci.yml` reusable through `workflow_call` so manual packaging validates the exact SHA with the same checks as daily CI.

`package-release.yml`: prepare (validate inputs/tag) → reusable CI → selected Windows and Android builders → one publisher. Stage files as short-lived artifacts, then publish a complete draft Release. Use GitHub's built-in token only in read-only preflight and the publisher; publication alone gets contents:write. Never run packaging on push/PR/tag events.

Use a small Python standard-library helper for input validation, metadata/checksum collection and guarded Release publication. Test tag/SHA conflicts, incomplete asset sets and published-release refusal with fake command responses; no test writes to GitHub.

Android signing accepts complete CI environment values or a local ignored key.properties. Local builds without keys retain internal debug-signing fallback; CI sets paprRequireReleaseSigning=true and fails without keys. CI verifies the resulting certificate against a public repository variable. Signing material lives in runner temp and is never cached/uploaded. A local PowerShell setup script prompts securely and sends Secrets through stdin, preserving an existing keystore; the owner chooses the key outside chat. No paid certificate or store integration.

Keep existing untracked android-release.yml content but disable its jobs with an explanatory guard; its useful ABI/NDK/signing steps feed the unified manual entry. Inherited legacy release/Homebrew jobs remain available only upstream. No deletion of inherited platform features.
