import java.util.Properties

plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

val signingProperties = Properties().apply {
    val propertiesFile = rootProject.file("key.properties")
    if (propertiesFile.isFile) {
        propertiesFile.inputStream().use { load(it) }
    }
}
fun signingValue(environmentName: String, propertyName: String): String? =
    System.getenv(environmentName) ?: signingProperties.getProperty(propertyName)

val releaseStoreFile = signingValue("PAPR_ANDROID_KEYSTORE_PATH", "storeFile")
val releaseStorePassword = signingValue("PAPR_ANDROID_STORE_PASSWORD", "storePassword")
val releaseKeyAlias = signingValue("PAPR_ANDROID_KEY_ALIAS", "keyAlias")
val releaseKeyPassword = signingValue("PAPR_ANDROID_KEY_PASSWORD", "keyPassword")
val signingValues = listOf(releaseStoreFile, releaseStorePassword, releaseKeyAlias, releaseKeyPassword)
val hasReleaseSigning = signingValues.all { !it.isNullOrEmpty() }
val requireReleaseSigning = project.findProperty("paprRequireReleaseSigning")?.toString() == "true"
if ((signingValues.any { it != null } || requireReleaseSigning) && !hasReleaseSigning) {
    throw GradleException("Complete Android release signing configuration is required; see README.")
}
if (hasReleaseSigning && !rootProject.file(releaseStoreFile!!).isFile) {
    throw GradleException("Configured Android release keystore does not exist.")
}

android {
    namespace = "com.papr.papr_mobile"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = "30.0.14904198"

    compileOptions {
        isCoreLibraryDesugaringEnabled = true
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    defaultConfig {
        // TODO: Specify your own unique Application ID (https://developer.android.com/studio/build/application-id.html).
        applicationId = "com.papr.papr_mobile"
        // You can update the following values to match your application needs.
        // For more information, see: https://flutter.dev/to/review-gradle-config.
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        if (hasReleaseSigning) {
            create("release") {
                storeFile = rootProject.file(releaseStoreFile!!)
                storePassword = releaseStorePassword
                keyAlias = releaseKeyAlias
                keyPassword = releaseKeyPassword
            }
        }
    }

    buildTypes {
        release {
            // Local internal builds may use Debug signing; CI requires a persistent key.
            signingConfig = signingConfigs.getByName(if (hasReleaseSigning) "release" else "debug")
        }
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

flutter {
    source = "../.."
}

// Run after Flutter's variant callback, before AGP finalizes the output codes.
// Directly distributed APKs share the requested code rather than Play ABI offsets.
android.applicationVariants.configureEach {
    if (buildType.name == "release") {
        outputs.forEach { output ->
            (output as com.android.build.gradle.api.ApkVariantOutput).versionCodeOverride = flutter.versionCode
        }
    }
}

dependencies {
    coreLibraryDesugaring("com.android.tools:desugar_jdk_libs:2.1.4")

    val media3Version = "1.10.1"
    implementation("androidx.media3:media3-exoplayer:$media3Version")
    implementation("androidx.media3:media3-session:$media3Version")
}

val repoRoot = rootDir.parentFile.parentFile
val bridgeDir = File(repoRoot, "crates/papr-flutter-bridge")
val targetDir = File(repoRoot, "target")
val rustTargetsByAbi = mapOf(
    "arm64-v8a" to "aarch64-linux-android",
    "armeabi-v7a" to "armv7-linux-androideabi",
    "x86" to "i686-linux-android",
    "x86_64" to "x86_64-linux-android"
)

// Narrow the cross-compiled ABIs for release packaging, e.g.
//   gradle -PpaprAndroidAbis=arm64-v8a,armeabi-v7a
// Otherwise match Flutter's target-platform selection, including emulator builds.
val flutterAbisByTarget = mapOf(
    "android-arm" to "armeabi-v7a",
    "android-arm64" to "arm64-v8a",
    "android-x86" to "x86",
    "android-x64" to "x86_64"
)
val flutterTargetAbis = (project.findProperty("target-platform") as String?)
    ?.split(',')
    ?.mapNotNull { flutterAbisByTarget[it.trim()] }
    .orEmpty()
val splitPerAbi = (project.findProperty("split-per-abi") as String?)?.toBoolean() == true
val requestedAbis = (project.findProperty("paprAndroidAbis") as String?)
    ?.split(',')
    ?.map(String::trim)
    ?.filter(String::isNotEmpty)
    .orEmpty()
    .ifEmpty { flutterTargetAbis }
if (requestedAbis.isNotEmpty()) {
    val unknownAbis = requestedAbis.filterNot(rustTargetsByAbi::containsKey)
    if (unknownAbis.isNotEmpty()) {
        throw GradleException(
            "paprAndroidAbis contains unsupported entries: ${unknownAbis.joinToString(", ")}; " +
                "supported: ${rustTargetsByAbi.keys.joinToString(", ")}"
        )
    }
    if (flutterTargetAbis.isNotEmpty() &&
        (requestedAbis.any { it !in flutterTargetAbis } ||
            (splitPerAbi && flutterTargetAbis.any { it !in requestedAbis }))) {
        throw GradleException("paprAndroidAbis must provide the same native targets as the selected Flutter package.")
    }
    // Filter every native dependency, not only the Rust copy tasks. Otherwise
    // plugin-only slices advertise ABIs that lack the Flutter engine/bridge.
    // AGP rejects defaultConfig filters together with ABI splits. Flutter's
    // split filters already constrain JNI dependencies for split packages.
    if (!splitPerAbi) {
        // Flutter 3.44 otherwise replaces these filters with its full default
        // ABI list after evaluation, reintroducing incomplete plugin slices.
        extensions.extraProperties.set("disable-abi-filtering", "true")
        android.defaultConfig.ndk.abiFilters.clear()
        android.defaultConfig.ndk.abiFilters.addAll(requestedAbis)
    }
}
val rustAbis =
    if (requestedAbis.isEmpty()) rustTargetsByAbi else rustTargetsByAbi.filterKeys { it in requestedAbis }

val staleAbiDirs = (rustTargetsByAbi.keys - rustAbis.keys)
    .filter { File(projectDir, "src/main/jniLibs/$it").listFiles().orEmpty().isNotEmpty() }
if (staleAbiDirs.isNotEmpty()) {
    logger.warn(
        "Native libraries for ${staleAbiDirs.joinToString(", ")} remain in jniLibs " +
            "and are excluded by this build's ABI filters."
    )
}

rustAbis.forEach { (androidAbi, rustTarget) ->
    val taskSuffix = androidAbi.replace("-", "")
    tasks.register<Exec>("buildRustBridge$taskSuffix") {
        group = "build"
        description = "Build papr-flutter-bridge for $rustTarget"
        workingDir = bridgeDir
        commandLine("cargo", "build", "--release", "--locked", "-p", "papr-flutter-bridge", "--target", rustTarget)
        environment("CARGO_TARGET_DIR", targetDir.absolutePath)
    }
    tasks.register<Copy>("copyRustBridge$taskSuffix") {
        group = "build"
        description = "Copy papr-flutter-bridge .so for $androidAbi"
        dependsOn("buildRustBridge$taskSuffix")
        from(targetDir.resolve("$rustTarget/release/libpapr_flutter_bridge.so"))
        into(file("src/main/jniLibs/$androidAbi"))
    }
}

tasks.register("buildRustBridge") {
    group = "build"
    description = "Build papr-flutter-bridge Rust library for Android ABIs"
    dependsOn(rustAbis.keys.map { "copyRustBridge${it.replace("-", "")}" })
}

tasks.named("preBuild").configure {
    dependsOn("buildRustBridge")
}
