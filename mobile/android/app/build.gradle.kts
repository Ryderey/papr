plugins {
    id("com.android.application")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

android {
    namespace = "com.papr.papr_mobile"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = "30.0.14904198"

    compileOptions {
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

    buildTypes {
        release {
            // TODO: Add your own signing config for the release build.
            // Signing with the debug keys for now, so `flutter run --release` works.
            signingConfig = signingConfigs.getByName("debug")
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

dependencies {
    val media3Version = "1.10.1"
    implementation("androidx.media3:media3-exoplayer:$media3Version")
    implementation("androidx.media3:media3-session:$media3Version")
}

val repoRoot = rootDir.parentFile.parentFile
val bridgeDir = File(repoRoot, "crates/papr-flutter-bridge")
val targetDir = File(repoRoot, "target")
val rustAbis = mapOf(
    "arm64-v8a" to "aarch64-linux-android",
    "armeabi-v7a" to "armv7-linux-androideabi",
    "x86" to "i686-linux-android",
    "x86_64" to "x86_64-linux-android"
)

rustAbis.forEach { (androidAbi, rustTarget) ->
    val taskSuffix = androidAbi.replace("-", "")
    tasks.register<Exec>("buildRustBridge$taskSuffix") {
        group = "build"
        description = "Build papr-flutter-bridge for $rustTarget"
        workingDir = bridgeDir
        commandLine("cargo", "build", "--release", "-p", "papr-flutter-bridge", "--target", rustTarget)
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
