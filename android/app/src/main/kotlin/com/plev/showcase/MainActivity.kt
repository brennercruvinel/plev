package com.plev.showcase

import com.google.androidgamesdk.GameActivity

/**
 * Host activity for the plev showcase.
 *
 * GameActivity loads the native library named by the `android.app.lib_name`
 * manifest meta-data (`showcase`) and calls the Rust `android_main` exported by
 * the showcase cdylib; winit owns the surface and event loop from there. There
 * is no Kotlin UI: every pixel is drawn on the GPU by the plev engine.
 */
class MainActivity : GameActivity() {
    companion object {
        init {
            System.loadLibrary("showcase")
        }
    }
}
