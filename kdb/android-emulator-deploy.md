---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-09
domain: build-android
---

# android emulator deploy

## resumo
φ rodando no android emulador (pixel 8, API 35, arm64-v8a) com showcase completo renderizando, quads, texto, layers, effects, input.

## stack de build
- **cargo-ndk** cross-compila `libφ.so` para `aarch64-linux-android`
- **gradle** empacota o APK com `MainActivity extends GameActivity` (java)
- **android-activity** crate v0.6.0 + **games-activity** aar 2.0.2 (versões devem coincidir)
- `android/` dir no repo: `settings.gradle`, `build.gradle`, `app/build.gradle`, `AndroidManifest.xml`, `MainActivity.java`, `styles.xml`

## comandos
```bash
cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs/ build --features android-game-activity
cd android && ./gradlew installDebug
adb shell am start -n com.φ.engine/.MainActivity
```

## problema crítico: swiftshader trava
- emulador com `hw.gpu.enabled=no` usa swiftshader (software vulkan)
- `device.create_render_pipeline()` trava **permanentemente** no swiftshader, nunca retorna
- **fix**: `hw.gpu.enabled=yes` + `hw.gpu.mode=host` no avd config (`~/.android/avd/Pixel_8.avd/config.ini`)
- com GPU host, adapter é apple m4 (via gfxstream), init completa em ~700ms

## lições
- `pollster::block_on()` em `resumed()` funciona normalmente com GPU host
- `cargo-apk` não serve, hardcoded para nativeactivity, não suporta workspaces
- `games-activity:3.0.5` causa `NoSuchMethodError`, android-activity 0.6.0 espera 2.0.2
- theme.appcompat obrigatório no styles.xml para evitar crash no oncreate
