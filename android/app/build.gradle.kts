// AGP 9 has built-in Kotlin support: no separate kotlin-android plugin.
plugins {
    alias(libs.plugins.android.application)
}

android {
    namespace = "com.plev.showcase"
    compileSdk = 35
    ndkVersion = "27.2.12479018"

    defaultConfig {
        applicationId = "com.plev.showcase"
        minSdk = 28
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        // Device arm64 + simulator/emulator x86_64; cargo-ndk fills jniLibs.
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    buildTypes {
        getByName("debug") {
            isDebuggable = true
        }
        getByName("release") {
            isMinifyEnabled = false
        }
    }

    // The Rust cdylib (libshowcase.so) is produced by cargo-ndk into jniLibs.
    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    implementation(libs.games.activity)
    implementation(libs.appcompat)
}
