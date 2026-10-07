# plevdev

```
plevdev/
├── .cargo/
│   └── config.toml
├── android/
│   ├── app/
│   │   ├── src/
│   │   │   └── main/
│   │   │       ├── kotlin/
│   │   │       │   └── com/
│   │   │       │       └── plev/
│   │   │       │           └── showcase/
│   │   │       │               └── MainActivity.kt
│   │   │       ├── res/
│   │   │       │   └── values/
│   │   │       │       └── themes.xml
│   │   │       └── AndroidManifest.xml
│   │   └── build.gradle.kts
│   ├── gradle/
│   │   ├── wrapper/
│   │   │   ├── gradle-wrapper.jar
│   │   │   └── gradle-wrapper.properties
│   │   └── libs.versions.toml
│   ├── build.gradle.kts
│   ├── build_android.sh
│   ├── gradle.properties
│   ├── gradlew
│   ├── gradlew.bat
│   └── settings.gradle.kts
├── boiler/
├── crates/
│   ├── claude-model/
│   │   ├── examples/
│   │   │   └── propose.rs
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── reply.rs
│   │   │   └── run.rs
│   │   ├── tests/
│   │   │   ├── cli.rs
│   │   │   └── fake-claude
│   │   └── Cargo.toml
│   ├── comps/
│   │   ├── examples/
│   │   │   └── editor/
│   │   │       ├── main.rs
│   │   │       └── render.rs
│   │   ├── src/
│   │   │   ├── action/
│   │   │   │   ├── badge.rs
│   │   │   │   ├── button.rs
│   │   │   │   ├── chip.rs
│   │   │   │   ├── icon_button.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── press.rs
│   │   │   │   └── submit_button.rs
│   │   │   ├── charts/
│   │   │   │   ├── area.rs
│   │   │   │   ├── axis.rs
│   │   │   │   ├── bars.rs
│   │   │   │   ├── donut.rs
│   │   │   │   ├── draw.rs
│   │   │   │   ├── line.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── tests_donut.rs
│   │   │   ├── content/
│   │   │   │   ├── avatar.rs
│   │   │   │   ├── card.rs
│   │   │   │   ├── code_block.rs
│   │   │   │   ├── empty_state.rs
│   │   │   │   ├── list.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── panel.rs
│   │   │   │   ├── separator.rs
│   │   │   │   ├── skeleton.rs
│   │   │   │   ├── stat.rs
│   │   │   │   ├── table.rs
│   │   │   │   └── tree.rs
│   │   │   ├── editor/
│   │   │   │   ├── config.rs
│   │   │   │   ├── input.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── tests_bidi.rs
│   │   │   │   ├── tests_large.rs
│   │   │   │   └── view.rs
│   │   │   ├── feedback/
│   │   │   │   ├── context_menu.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── modal.rs
│   │   │   │   ├── progress.rs
│   │   │   │   ├── scrollbar.rs
│   │   │   │   ├── spinner.rs
│   │   │   │   ├── toast.rs
│   │   │   │   └── tooltip.rs
│   │   │   ├── form/
│   │   │   │   ├── select/
│   │   │   │   │   └── panel.rs
│   │   │   │   ├── text_input/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── tests_bidi.rs
│   │   │   │   │   │   ├── tests_buffer.rs
│   │   │   │   │   │   ├── tests_clipboard.rs
│   │   │   │   │   │   ├── tests_component.rs
│   │   │   │   │   │   └── tests_ime.rs
│   │   │   │   │   ├── buffer.rs
│   │   │   │   │   ├── component.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── tests.rs
│   │   │   │   │   └── visual.rs
│   │   │   │   ├── checkbox.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── select.rs
│   │   │   │   ├── slider.rs
│   │   │   │   ├── switch.rs
│   │   │   │   └── text_field.rs
│   │   │   ├── icons/
│   │   │   │   ├── data.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── svg.rs
│   │   │   │   └── tests.rs
│   │   │   ├── nav/
│   │   │   │   ├── sidebar/
│   │   │   │   │   ├── input.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── render.rs
│   │   │   │   ├── breadcrumb.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── nav_link.rs
│   │   │   │   ├── panel_header.rs
│   │   │   │   ├── split_pane.rs
│   │   │   │   └── tabs.rs
│   │   │   ├── overlay/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── types.rs
│   │   │   ├── tests/
│   │   │   │   ├── widgets/
│   │   │   │   │   ├── semantics/
│   │   │   │   │   │   ├── action.rs
│   │   │   │   │   │   ├── content.rs
│   │   │   │   │   │   ├── feedback.rs
│   │   │   │   │   │   ├── form.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── nav.rs
│   │   │   │   │   │   └── roving.rs
│   │   │   │   │   ├── action.rs
│   │   │   │   │   ├── content.rs
│   │   │   │   │   ├── feedback.rs
│   │   │   │   │   ├── fixtures.rs
│   │   │   │   │   ├── form.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── nav.rs
│   │   │   │   ├── components.rs
│   │   │   │   ├── focus.rs
│   │   │   │   ├── matrix.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── wake.rs
│   │   │   ├── core.rs
│   │   │   ├── graph.rs
│   │   │   ├── lib.rs
│   │   │   ├── recipe.rs
│   │   │   ├── shell.rs
│   │   │   └── wake.rs
│   │   ├── tests/
│   │   │   └── editor_large_bench.rs
│   │   └── Cargo.toml
│   ├── engine/
│   │   ├── assets/
│   │   │   ├── fonts/
│   │   │   │   ├── codicons.ttf
│   │   │   │   ├── InclusiveSans-Bold.ttf
│   │   │   │   ├── InclusiveSans-Light.ttf
│   │   │   │   ├── InclusiveSans-Medium.ttf
│   │   │   │   ├── InclusiveSans-Regular.ttf
│   │   │   │   ├── InclusiveSans-SemiBold.ttf
│   │   │   │   ├── Inter-Medium.ttf
│   │   │   │   ├── Inter-Regular.ttf
│   │   │   │   ├── Inter-SemiBold.ttf
│   │   │   │   ├── LICENSE-codicons-CC-BY-4.0.txt
│   │   │   │   ├── LICENSE-InclusiveSans-OFL.txt
│   │   │   │   ├── LICENSE-Inter-OFL.txt
│   │   │   │   ├── LICENSE-Noto-OFL.txt
│   │   │   │   ├── LICENSE-NotoColorEmoji-OFL.txt
│   │   │   │   ├── NotoColorEmoji-CBDT.ttf
│   │   │   │   ├── NotoSansArabic-Bold.ttf
│   │   │   │   ├── NotoSansArabic-Regular.ttf
│   │   │   │   ├── NotoSansDevanagari-Bold.ttf
│   │   │   │   ├── NotoSansDevanagari-Regular.ttf
│   │   │   │   ├── NotoSansHebrew-Bold.ttf
│   │   │   │   ├── NotoSansHebrew-Regular.ttf
│   │   │   │   ├── NotoSansJP-Bold.otf
│   │   │   │   ├── NotoSansJP-Regular.otf
│   │   │   │   ├── NotoSansKR-Bold.otf
│   │   │   │   ├── NotoSansKR-Regular.otf
│   │   │   │   ├── NotoSansSC-Bold.otf
│   │   │   │   └── NotoSansSC-Regular.otf
│   │   │   ├── logo-plev.png
│   │   │   ├── plev-logo-icon.svg
│   │   │   └── plev.svg
│   │   ├── benches/
│   │   │   └── scene_build.rs
│   │   ├── examples/
│   │   │   ├── builder/
│   │   │   │   ├── main.rs
│   │   │   │   ├── render.rs
│   │   │   │   └── ui.rs
│   │   │   ├── charts/
│   │   │   │   ├── charts.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── palette.rs
│   │   │   │   ├── render.rs
│   │   │   │   ├── state.rs
│   │   │   │   └── submit.rs
│   │   │   ├── counter/
│   │   │   │   ├── lifecycle.rs
│   │   │   │   ├── main.rs
│   │   │   │   └── render.rs
│   │   │   ├── input/
│   │   │   │   ├── gpu.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── palette.rs
│   │   │   │   ├── render.rs
│   │   │   │   └── state.rs
│   │   │   ├── layers/
│   │   │   │   ├── main.rs
│   │   │   │   ├── palette.rs
│   │   │   │   ├── render.rs
│   │   │   │   └── ui.rs
│   │   │   ├── lot2monsters/
│   │   │   │   └── main.rs
│   │   │   ├── lottie_player/
│   │   │   │   └── main.rs
│   │   │   ├── mobile_input/
│   │   │   │   ├── main.rs
│   │   │   │   ├── palette.rs
│   │   │   │   ├── render.rs
│   │   │   │   ├── state.rs
│   │   │   │   └── ui.rs
│   │   │   ├── monster_player/
│   │   │   │   └── main.rs
│   │   │   ├── scene3d/
│   │   │   │   ├── components/
│   │   │   │   │   ├── chip.rs
│   │   │   │   │   ├── hud_panel.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── stat_card.rs
│   │   │   │   ├── scene3d/
│   │   │   │   │   ├── camera.rs
│   │   │   │   │   ├── geometry.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── pipeline.rs
│   │   │   │   │   ├── scene.rs
│   │   │   │   │   └── shader.wgsl
│   │   │   │   ├── main.rs
│   │   │   │   └── theme.rs
│   │   │   ├── sdf_shapes/
│   │   │   │   ├── cards_row1.rs
│   │   │   │   ├── cards_row2.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── render.rs
│   │   │   │   └── shapes.rs
│   │   │   ├── snake/
│   │   │   │   ├── main.rs
│   │   │   │   ├── rendering.rs
│   │   │   │   ├── state.rs
│   │   │   │   └── ui.rs
│   │   │   ├── svg2monster/
│   │   │   │   └── main.rs
│   │   │   ├── text/
│   │   │   │   ├── gpu.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── palette.rs
│   │   │   │   ├── render.rs
│   │   │   │   └── state.rs
│   │   │   ├── text_probe/
│   │   │   │   └── main.rs
│   │   │   └── visual/
│   │   │       ├── main.rs
│   │   │       └── scene.rs
│   │   ├── src/
│   │   │   ├── accessibility/
│   │   │   │   ├── action/
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── present/
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── tree/
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── action.rs
│   │   │   │   ├── bridge.rs
│   │   │   │   ├── focus.rs
│   │   │   │   ├── ids.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── motion.rs
│   │   │   │   ├── present.rs
│   │   │   │   ├── role.rs
│   │   │   │   └── tree.rs
│   │   │   ├── actions/
│   │   │   │   ├── context.rs
│   │   │   │   ├── keymap.rs
│   │   │   │   ├── keystroke.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── shortcuts.rs
│   │   │   │   └── winit_keys.rs
│   │   │   ├── animation/
│   │   │   │   ├── easing.rs
│   │   │   │   ├── keyframe.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── spring.rs
│   │   │   │   ├── tests_clock.rs
│   │   │   │   ├── tests_easing.rs
│   │   │   │   ├── tests_keyframe.rs
│   │   │   │   ├── tests_motion.rs
│   │   │   │   ├── tests_spring.rs
│   │   │   │   ├── tests_tween.rs
│   │   │   │   └── tween.rs
│   │   │   ├── builder/
│   │   │   │   ├── element/
│   │   │   │   │   ├── builders.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── tests/
│   │   │   │   │   ├── emit.rs
│   │   │   │   │   ├── layout.rs
│   │   │   │   │   └── styles.rs
│   │   │   │   ├── emit.rs
│   │   │   │   ├── layout_pipeline.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── render.rs
│   │   │   │   ├── style.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── tests_intent.rs
│   │   │   │   ├── traits.rs
│   │   │   │   └── types.rs
│   │   │   ├── component/
│   │   │   │   ├── lifecycle_impl.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── tests.rs
│   │   │   ├── compositor/
│   │   │   │   ├── layer/
│   │   │   │   │   ├── geometry.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── texture.rs
│   │   │   │   ├── binding.rs
│   │   │   │   ├── clip.rs
│   │   │   │   ├── drawing.rs
│   │   │   │   ├── layer_ops.rs
│   │   │   │   ├── memory.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── scene.rs
│   │   │   │   ├── sequence.rs
│   │   │   │   ├── stats.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── vertex.rs
│   │   │   ├── effects/
│   │   │   │   ├── apply.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── pipelines.rs
│   │   │   │   ├── processor.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── types.rs
│   │   │   ├── gpu/
│   │   │   │   ├── shaders/
│   │   │   │   │   ├── 3Dscene.wgsl
│   │   │   │   │   ├── backdrop.wgsl
│   │   │   │   │   ├── blur.wgsl
│   │   │   │   │   ├── composite.wgsl
│   │   │   │   │   ├── image.wgsl
│   │   │   │   │   ├── quad.wgsl
│   │   │   │   │   ├── rect_sdf.wgsl
│   │   │   │   │   ├── shadow.wgsl
│   │   │   │   │   ├── shadow_analytic.wgsl
│   │   │   │   │   └── text.wgsl
│   │   │   │   ├── config.rs
│   │   │   │   ├── context.rs
│   │   │   │   ├── image.rs
│   │   │   │   ├── loss.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── pipelines.rs
│   │   │   │   ├── recover.rs
│   │   │   │   ├── surface.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── texture_pool.rs
│   │   │   │   ├── utils.rs
│   │   │   │   └── vec.rs
│   │   │   ├── headless/
│   │   │   │   ├── svg/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── shape.rs
│   │   │   │   │   ├── tests.rs
│   │   │   │   │   ├── text.rs
│   │   │   │   │   └── write.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── readback.rs
│   │   │   │   └── tests.rs
│   │   │   ├── hot_reload/
│   │   │   │   ├── literals/
│   │   │   │   │   ├── write/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   └── write.rs
│   │   │   │   ├── literals.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── narrate.rs
│   │   │   │   ├── shader.rs
│   │   │   │   ├── source.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── view_source.rs
│   │   │   ├── input/
│   │   │   │   ├── gesture/
│   │   │   │   │   ├── handlers.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── recognizer.rs
│   │   │   │   │   ├── state.rs
│   │   │   │   │   ├── tests.rs
│   │   │   │   │   ├── tests_basic.rs
│   │   │   │   │   └── tests_complex.rs
│   │   │   │   ├── touch/
│   │   │   │   │   └── synth.rs
│   │   │   │   ├── dispatch.rs
│   │   │   │   ├── drop.rs
│   │   │   │   ├── handlers.rs
│   │   │   │   ├── hit_test.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── scroll.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── tests_hit.rs
│   │   │   │   ├── touch.rs
│   │   │   │   └── types.rs
│   │   │   ├── layout/
│   │   │   │   ├── tests/
│   │   │   │   │   ├── flex.rs
│   │   │   │   │   ├── measure.rs
│   │   │   │   │   └── percent.rs
│   │   │   │   ├── engine.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── tests_perf.rs
│   │   │   │   └── types.rs
│   │   │   ├── narrate_runtime/
│   │   │   │   ├── tests/
│   │   │   │   │   ├── extraction_tests.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── extraction.rs
│   │   │   │   ├── keywords.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── modifiers.rs
│   │   │   │   ├── parser.rs
│   │   │   │   └── tokenizer.rs
│   │   │   ├── path/
│   │   │   │   ├── builder.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tessellation.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── types.rs
│   │   │   ├── perf/
│   │   │   │   ├── gpu_time.rs
│   │   │   │   ├── hud.rs
│   │   │   │   ├── memory.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── stages.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── tests_stages.rs
│   │   │   │   └── window.rs
│   │   │   ├── platform/
│   │   │   │   ├── capture/
│   │   │   │   │   ├── ffmpeg/
│   │   │   │   │   │   ├── args.rs
│   │   │   │   │   │   └── child.rs
│   │   │   │   │   └── ffmpeg.rs
│   │   │   │   ├── links/
│   │   │   │   │   ├── source.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── menu/
│   │   │   │   │   └── native.rs
│   │   │   │   ├── print/
│   │   │   │   │   └── cups.rs
│   │   │   │   ├── tray/
│   │   │   │   │   └── native.rs
│   │   │   │   ├── alerts.rs
│   │   │   │   ├── capture.rs
│   │   │   │   ├── files.rs
│   │   │   │   ├── ime.rs
│   │   │   │   ├── lifecycle.rs
│   │   │   │   ├── links.rs
│   │   │   │   ├── menu.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── notify.rs
│   │   │   │   ├── print.rs
│   │   │   │   └── tray.rs
│   │   │   ├── signal/
│   │   │   │   ├── api.rs
│   │   │   │   ├── execution.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── runtime.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── tests_advanced.rs
│   │   │   ├── text/
│   │   │   │   ├── atlas.rs
│   │   │   │   ├── backend.rs
│   │   │   │   ├── cache.rs
│   │   │   │   ├── cluster_cache.rs
│   │   │   │   ├── fonts.rs
│   │   │   │   ├── measure.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── probe.rs
│   │   │   │   ├── system.rs
│   │   │   │   ├── tests_measure.rs
│   │   │   │   ├── tests_raster.rs
│   │   │   │   └── vertex.rs
│   │   │   ├── theme/
│   │   │   │   ├── button.rs
│   │   │   │   ├── color_space.rs
│   │   │   │   ├── core.rs
│   │   │   │   ├── hoff.rs
│   │   │   │   ├── hoff_core.rs
│   │   │   │   ├── intent.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── scales.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── tests_contrast.rs
│   │   │   │   ├── tests_oklch.rs
│   │   │   │   ├── tests_scales.rs
│   │   │   │   └── tokens.rs
│   │   │   ├── view/
│   │   │   │   ├── context.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tests.rs
│   │   │   │   ├── trait_def.rs
│   │   │   │   └── views.rs
│   │   │   ├── window/
│   │   │   │   ├── events.rs
│   │   │   │   ├── frame.rs
│   │   │   │   ├── host.rs
│   │   │   │   ├── hot_reload.rs
│   │   │   │   ├── launch.rs
│   │   │   │   ├── lifecycle.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── render.rs
│   │   │   │   ├── render_passes.rs
│   │   │   │   ├── sleep.rs
│   │   │   │   └── windows.rs
│   │   │   ├── clipboard.rs
│   │   │   ├── color.rs
│   │   │   ├── error.rs
│   │   │   ├── graph.rs
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   └── blur_at_scale_1.png
│   │   │   ├── text_corpus/
│   │   │   │   ├── 01-latin-diacritics.txt
│   │   │   │   ├── 02-latin-ligatures-kerning.txt
│   │   │   │   ├── 03-combining-marks.txt
│   │   │   │   ├── 04-arabic.txt
│   │   │   │   ├── 05-hebrew.txt
│   │   │   │   ├── 06-devanagari.txt
│   │   │   │   ├── 07-chinese.txt
│   │   │   │   ├── 08-japanese.txt
│   │   │   │   ├── 09-korean.txt
│   │   │   │   ├── 10-emoji.txt
│   │   │   │   ├── 11-bidi-mixed.txt
│   │   │   │   └── README.txt
│   │   │   ├── alerts.rs
│   │   │   ├── arc_sync_guard.rs
│   │   │   ├── asset_cost.rs
│   │   │   ├── blur_scale.rs
│   │   │   ├── capture.rs
│   │   │   ├── device_loss.rs
│   │   │   ├── fake-ffmpeg
│   │   │   ├── fake-ffmpeg-mute
│   │   │   ├── fake-ffmpeg-refuses
│   │   │   ├── fake-lp
│   │   │   ├── fake-lpstat
│   │   │   ├── fake-lpstat-none
│   │   │   ├── file_hygiene_guard.rs
│   │   │   ├── headless_determinism.rs
│   │   │   ├── links.rs
│   │   │   ├── menu.rs
│   │   │   ├── notify.rs
│   │   │   ├── print.rs
│   │   │   ├── sleep_wake.rs
│   │   │   ├── text_audit.rs
│   │   │   ├── text_audit.txt
│   │   │   ├── text_drawn_extent.rs
│   │   │   ├── text_raster_pixels.rs
│   │   │   ├── text_style_guard.rs
│   │   │   ├── tray.rs
│   │   │   ├── wgsl.rs
│   │   │   └── windows.rs
│   │   ├── build.rs
│   │   └── Cargo.toml
│   ├── git/
│   │   ├── src/
│   │   │   ├── client.rs
│   │   │   ├── diff.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── repo.rs
│   │   │   ├── status.rs
│   │   │   └── types.rs
│   │   ├── tests/
│   │   │   ├── log_cost.rs
│   │   │   └── real_repo.rs
│   │   └── Cargo.toml
│   ├── ide/
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   └── lab/
│   │   │   │       ├── answer.rs
│   │   │   │       ├── args.rs
│   │   │   │       ├── asked.rs
│   │   │   │       ├── bench.rs
│   │   │   │       ├── finish.rs
│   │   │   │       ├── gitarm.rs
│   │   │   │       ├── host.rs
│   │   │   │       ├── lab.rs
│   │   │   │       ├── main.rs
│   │   │   │       ├── refused.rs
│   │   │   │       ├── report.rs
│   │   │   │       ├── review.rs
│   │   │   │       ├── screenshot.rs
│   │   │   │       ├── scripted.rs
│   │   │   │       └── story.rs
│   │   │   ├── generated/
│   │   │   │   ├── claude.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── row.rs
│   │   │   │   └── scope.rs
│   │   │   ├── views/
│   │   │   │   ├── workspace/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── overlays.rs
│   │   │   │   │   │   ├── panels.rs
│   │   │   │   │   │   ├── scrolling.rs
│   │   │   │   │   │   └── wake.rs
│   │   │   │   │   ├── input.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── overlays.rs
│   │   │   │   │   ├── render.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── commit_form.rs
│   │   │   │   ├── diff_view.rs
│   │   │   │   ├── header.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── multi_stack_view.rs
│   │   │   │   ├── sidebar.rs
│   │   │   │   └── unassigned_view.rs
│   │   │   ├── workbench/
│   │   │   │   ├── asking.rs
│   │   │   │   ├── column.rs
│   │   │   │   ├── execute.rs
│   │   │   │   ├── generated.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── notice.rs
│   │   │   │   ├── person.rs
│   │   │   │   ├── project.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── truth.rs
│   │   │   ├── actions.rs
│   │   │   ├── adapters.rs
│   │   │   ├── agent.rs
│   │   │   ├── ids.rs
│   │   │   ├── keymap.json
│   │   │   ├── keys.rs
│   │   │   ├── lab_repo.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   ├── pace.rs
│   │   │   ├── record.rs
│   │   │   ├── recording.rs
│   │   │   ├── renderer.rs
│   │   │   ├── session.rs
│   │   │   ├── status.rs
│   │   │   └── watcher.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   └── sessions/
│   │   │   │       ├── a2ppa.marks
│   │   │   │       ├── a2ppa.monster
│   │   │   │       ├── bug84.marks
│   │   │   │       └── bug84.monster
│   │   │   ├── rerun/
│   │   │   │   ├── harness.rs
│   │   │   │   ├── inflight.rs
│   │   │   │   ├── lab.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── marks.rs
│   │   │   │   ├── report.rs
│   │   │   │   └── toy.rs
│   │   │   ├── support/
│   │   │   │   └── mod.rs
│   │   │   ├── agent_port.rs
│   │   │   ├── claude.rs
│   │   │   ├── human_agent.rs
│   │   │   ├── lab.rs
│   │   │   ├── lab_review.rs
│   │   │   ├── line.rs
│   │   │   ├── native_confirm.rs
│   │   │   ├── observe_cost.rs
│   │   │   ├── record.rs
│   │   │   ├── review.rs
│   │   │   ├── review_acts.rs
│   │   │   ├── surface.rs
│   │   │   └── uir_fixture.rs
│   │   └── Cargo.toml
│   ├── inspect/
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   └── plev_inspect.rs
│   │   │   ├── text/
│   │   │   │   ├── chain.rs
│   │   │   │   └── mod.rs
│   │   │   ├── control.rs
│   │   │   ├── describe.rs
│   │   │   ├── index.rs
│   │   │   ├── lib.rs
│   │   │   ├── summary.rs
│   │   │   ├── timeline.rs
│   │   │   └── why.rs
│   │   ├── tests/
│   │   │   └── model/
│   │   │       ├── apps.rs
│   │   │       ├── cli.rs
│   │   │       ├── control.rs
│   │   │       ├── export.rs
│   │   │       ├── main.rs
│   │   │       ├── readonly.rs
│   │   │       ├── text.rs
│   │   │       ├── thesis.rs
│   │   │       ├── timeline.rs
│   │   │       └── why.rs
│   │   └── Cargo.toml
│   ├── inspector/
│   │   ├── src/
│   │   │   ├── fit.rs
│   │   │   ├── input.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   ├── panel.rs
│   │   │   ├── render.rs
│   │   │   ├── strip.rs
│   │   │   ├── view.rs
│   │   │   └── window.rs
│   │   ├── tests/
│   │   │   └── ui/
│   │   │       ├── accept.rs
│   │   │       ├── main.rs
│   │   │       ├── model.rs
│   │   │       ├── privacy.rs
│   │   │       ├── readonly.rs
│   │   │       ├── support.rs
│   │   │       └── viewport.rs
│   │   └── Cargo.toml
│   ├── llm/
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   └── llm_bench.rs
│   │   │   ├── burn/
│   │   │   │   ├── cache.rs
│   │   │   │   ├── load.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── model.rs
│   │   │   │   └── rope.rs
│   │   │   ├── candle.rs
│   │   │   ├── chat.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── plev.rs
│   │   │   ├── stream.rs
│   │   │   └── text.rs
│   │   ├── tests/
│   │   │   ├── plev.rs
│   │   │   └── real.rs
│   │   ├── Cargo.lock
│   │   ├── Cargo.toml
│   │   └── NOTICE
│   ├── lot/
│   │   ├── benches/
│   │   │   └── convert.rs
│   │   ├── src/
│   │   │   ├── cnv.rs
│   │   │   ├── gem.rs
│   │   │   ├── kfr.rs
│   │   │   ├── lib.rs
│   │   │   ├── mdl.rs
│   │   │   └── rnd.rs
│   │   └── Cargo.toml
│   ├── macros/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── monster/
│   │   ├── benches/
│   │   │   └── codec.rs
│   │   ├── fixtures/
│   │   │   ├── golden_v0_minimal.monster
│   │   │   └── golden_v0_ops.monster
│   │   ├── src/
│   │   │   ├── tests/
│   │   │   │   ├── asset_path.rs
│   │   │   │   ├── discover.rs
│   │   │   │   ├── discover_ops.rs
│   │   │   │   ├── easing.rs
│   │   │   │   ├── golden.rs
│   │   │   │   ├── ir.rs
│   │   │   │   ├── lower.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ops.rs
│   │   │   │   ├── optimize.rs
│   │   │   │   ├── optimize_pipe.rs
│   │   │   │   ├── play.rs
│   │   │   │   ├── play_ops.rs
│   │   │   │   ├── prop.rs
│   │   │   │   ├── quant.rs
│   │   │   │   ├── read.rs
│   │   │   │   ├── read_malformed.rs
│   │   │   │   └── write.rs
│   │   │   ├── asset_path.rs
│   │   │   ├── container.rs
│   │   │   ├── discover.rs
│   │   │   ├── discover_fit.rs
│   │   │   ├── easing.rs
│   │   │   ├── ir.rs
│   │   │   ├── lib.rs
│   │   │   ├── lower.rs
│   │   │   ├── optimize.rs
│   │   │   ├── play.rs
│   │   │   ├── play_eval.rs
│   │   │   ├── quant.rs
│   │   │   ├── raise.rs
│   │   │   ├── read.rs
│   │   │   ├── read_sec.rs
│   │   │   ├── validate.rs
│   │   │   └── write.rs
│   │   ├── tests/
│   │   │   ├── gate_bytes/
│   │   │   │   ├── dense.rs
│   │   │   │   ├── girl.rs
│   │   │   │   ├── kfs.rs
│   │   │   │   ├── lot.rs
│   │   │   │   └── main.rs
│   │   │   ├── gate_mode_b.rs
│   │   │   └── raise.rs
│   │   └── Cargo.toml
│   ├── narrate/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   └── integration.rs
│   │   └── Cargo.toml
│   ├── narrate-macro/
│   │   ├── src/
│   │   │   ├── codegen/
│   │   │   │   ├── generators.rs
│   │   │   │   ├── interpolation.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── tests.rs
│   │   │   ├── parse/
│   │   │   │   ├── keywords/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── suggest.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── tests/
│   │   │   │   │   ├── errors.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── block_item.rs
│   │   │   │   ├── element.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── modifier.rs
│   │   │   │   └── value.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── parser/
│   │   ├── benches/
│   │   │   └── transpile.rs
│   │   ├── examples/
│   │   │   ├── preview.rs
│   │   │   └── transpile.rs
│   │   ├── fixtures/
│   │   │   ├── figma/
│   │   │   │   ├── expected.ir.json
│   │   │   │   ├── expected.report.json
│   │   │   │   ├── expected.rs
│   │   │   │   └── settings.json
│   │   │   ├── gpui/
│   │   │   │   ├── expected.ir.json
│   │   │   │   ├── expected.report.json
│   │   │   │   ├── expected.rs
│   │   │   │   └── separator.rs
│   │   │   ├── html/
│   │   │   │   ├── expected.ir.json
│   │   │   │   ├── expected.report.json
│   │   │   │   ├── expected.rs
│   │   │   │   └── note-card.html
│   │   │   └── react/
│   │   │       ├── expected.ir.json
│   │   │       ├── expected.report.json
│   │   │       ├── expected.rs
│   │   │       ├── index.tsx
│   │   │       ├── note-card-variables.sass
│   │   │       └── NoteCard.module.sass
│   │   ├── src/
│   │   │   ├── emit_ir/
│   │   │   │   ├── control.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── props.rs
│   │   │   ├── resolve_figma/
│   │   │   │   ├── look.rs
│   │   │   │   └── mod.rs
│   │   │   ├── resolve_html/
│   │   │   │   ├── frame.rs
│   │   │   │   └── mod.rs
│   │   │   ├── css_map.rs
│   │   │   ├── emit.rs
│   │   │   ├── figma.rs
│   │   │   ├── gpui.rs
│   │   │   ├── gpui_ir.rs
│   │   │   ├── html.rs
│   │   │   ├── html_map.rs
│   │   │   ├── ir.rs
│   │   │   ├── lib.rs
│   │   │   ├── report.rs
│   │   │   ├── resolve_gpui.rs
│   │   │   ├── resolve_react.rs
│   │   │   ├── sass.rs
│   │   │   ├── tokens.rs
│   │   │   └── tsx.rs
│   │   ├── tests/
│   │   │   ├── support/
│   │   │   │   ├── checks.rs
│   │   │   │   └── mod.rs
│   │   │   ├── emitted_compiles.rs
│   │   │   ├── figma.rs
│   │   │   ├── golden.rs
│   │   │   ├── html.rs
│   │   │   ├── ir.rs
│   │   │   └── pixels.rs
│   │   └── Cargo.toml
│   ├── prime/
│   │   ├── src/
│   │   │   ├── sim/
│   │   │   │   ├── coherence.rs
│   │   │   │   ├── grid.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── params.rs
│   │   │   │   ├── particle.rs
│   │   │   │   ├── physics.rs
│   │   │   │   ├── primes.rs
│   │   │   │   └── rng.rs
│   │   │   ├── color.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   └── scene.rs
│   │   ├── Cargo.toml
│   │   ├── index.html
│   │   └── Trunk.toml
│   ├── propose/
│   │   ├── src/
│   │   │   ├── compose/
│   │   │   │   ├── chain.rs
│   │   │   │   ├── composite.rs
│   │   │   │   ├── detect.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── offer.rs
│   │   │   ├── review/
│   │   │   │   └── tests.rs
│   │   │   ├── adapt.rs
│   │   │   ├── change.rs
│   │   │   ├── error.rs
│   │   │   ├── ids.rs
│   │   │   ├── lib.rs
│   │   │   ├── list.rs
│   │   │   ├── llm.rs
│   │   │   ├── ops.rs
│   │   │   ├── parts.rs
│   │   │   ├── prompt.rs
│   │   │   ├── reshape.rs
│   │   │   ├── review.rs
│   │   │   ├── row.rs
│   │   │   ├── rules.rs
│   │   │   └── words.rs
│   │   ├── tests/
│   │   │   ├── compose/
│   │   │   │   ├── chain.rs
│   │   │   │   ├── declare.rs
│   │   │   │   ├── desk.rs
│   │   │   │   ├── detect.rs
│   │   │   │   ├── main.rs
│   │   │   │   └── offer.rs
│   │   │   ├── support/
│   │   │   │   └── mod.rs
│   │   │   ├── adapt.rs
│   │   │   ├── change.rs
│   │   │   ├── llm.rs
│   │   │   ├── llm_change.rs
│   │   │   ├── media.rs
│   │   │   └── rules.rs
│   │   └── Cargo.toml
│   ├── rec/
│   │   ├── src/
│   │   │   ├── scene/
│   │   │   │   ├── asset.rs
│   │   │   │   ├── bytes.rs
│   │   │   │   ├── mask.rs
│   │   │   │   ├── mirror.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── node.rs
│   │   │   │   ├── send.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── tests_mask.rs
│   │   │   ├── bound.rs
│   │   │   ├── error.rs
│   │   │   ├── event.rs
│   │   │   ├── format.rs
│   │   │   ├── hook.rs
│   │   │   ├── lib.rs
│   │   │   ├── limits.rs
│   │   │   ├── payload.rs
│   │   │   ├── persist.rs
│   │   │   ├── recorder.rs
│   │   │   ├── ring.rs
│   │   │   ├── screen.rs
│   │   │   ├── session.rs
│   │   │   ├── state.rs
│   │   │   ├── track.rs
│   │   │   ├── visual.rs
│   │   │   └── weigh.rs
│   │   ├── tests/
│   │   │   ├── session/
│   │   │   │   ├── apps.rs
│   │   │   │   ├── bound.rs
│   │   │   │   ├── file.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── measure.rs
│   │   │   │   ├── replay.rs
│   │   │   │   ├── ring.rs
│   │   │   │   ├── secret.rs
│   │   │   │   ├── semantic.rs
│   │   │   │   ├── thesis.rs
│   │   │   │   └── visual.rs
│   │   │   ├── panic.rs
│   │   │   ├── scene.rs
│   │   │   └── scene_probe.rs
│   │   ├── build.rs
│   │   └── Cargo.toml
│   ├── rope/
│   │   ├── benches/
│   │   │   └── edit.rs
│   │   ├── src/
│   │   │   ├── document.rs
│   │   │   ├── history.rs
│   │   │   ├── lib.rs
│   │   │   ├── movement.rs
│   │   │   ├── selection.rs
│   │   │   ├── tests_bidi.rs
│   │   │   └── transaction.rs
│   │   ├── tests/
│   │   │   └── properties.rs
│   │   └── Cargo.toml
│   ├── sem/
│   │   ├── examples/
│   │   │   └── commit.rs
│   │   ├── src/
│   │   │   ├── builder.rs
│   │   │   ├── check.rs
│   │   │   ├── decl.rs
│   │   │   ├── diff.rs
│   │   │   ├── dispatch.rs
│   │   │   ├── error.rs
│   │   │   ├── hash.rs
│   │   │   ├── id.rs
│   │   │   ├── lib.rs
│   │   │   ├── name.rs
│   │   │   ├── node.rs
│   │   │   ├── receipt.rs
│   │   │   ├── redact.rs
│   │   │   ├── request.rs
│   │   │   ├── role.rs
│   │   │   ├── snapshot.rs
│   │   │   ├── text.rs
│   │   │   ├── validate.rs
│   │   │   └── value.rs
│   │   ├── tests/
│   │   │   ├── builder.rs
│   │   │   ├── check.rs
│   │   │   ├── diff.rs
│   │   │   ├── dispatch.rs
│   │   │   ├── id.rs
│   │   │   ├── receipt.rs
│   │   │   ├── request.rs
│   │   │   ├── snapshot.rs
│   │   │   ├── value.rs
│   │   │   └── wire.rs
│   │   └── Cargo.toml
│   ├── showcase/
│   │   ├── examples/
│   │   │   ├── demo/
│   │   │   │   ├── agent.rs
│   │   │   │   ├── main.rs
│   │   │   │   └── shell.rs
│   │   │   ├── measure/
│   │   │   │   ├── harness.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── probe_cost.rs
│   │   │   │   ├── report.rs
│   │   │   │   └── scenarios.rs
│   │   │   ├── mirror/
│   │   │   │   ├── main.rs
│   │   │   │   └── show.rs
│   │   │   └── regress/
│   │   │       ├── capture/
│   │   │       │   ├── summary.rs
│   │   │       │   └── tests.rs
│   │   │       ├── report/
│   │   │       │   ├── describe.rs
│   │   │       │   ├── json.rs
│   │   │       │   └── text.rs
│   │   │       ├── baseline.rs
│   │   │       ├── capture.rs
│   │   │       ├── classify.rs
│   │   │       ├── compare.rs
│   │   │       ├── determinism.rs
│   │   │       ├── hash.rs
│   │   │       ├── main.rs
│   │   │       ├── pixels.rs
│   │   │       ├── report.rs
│   │   │       ├── requested.rs
│   │   │       ├── run.rs
│   │   │       ├── scene.rs
│   │   │       ├── state.rs
│   │   │       └── tests.rs
│   │   ├── fixtures/
│   │   │   ├── demo/
│   │   │   │   ├── silences-demo-claude.monster
│   │   │   │   └── silences-demo.monster
│   │   │   ├── determinism/
│   │   │   │   └── requested.json
│   │   │   ├── ir/
│   │   │   │   └── silences-review-claude.ir.json
│   │   │   └── regress/
│   │   │       ├── accordion@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── accordion@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── alert-dialog@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── alert-dialog@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── alert@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── alert@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── aspect-ratio@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── aspect-ratio@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── attachment@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── attachment@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── avatar@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── avatar@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── badge@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── badge@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── breadcrumb@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── breadcrumb@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── bubble@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── bubble@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── builder@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── builder@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── button-group@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── button-group@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── button@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── button@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── calendar@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── calendar@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── capture@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── capture@hoff@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── card@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── card@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── carousel@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── carousel@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── chart@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── chart@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── checkbox@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── checkbox@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── chip@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── chip@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── code-block@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── code-block@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── collapsible@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── collapsible@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── combobox@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── combobox@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── command@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── command@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── context-menu@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── context-menu@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── data-table@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── data-table@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── date-picker@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── date-picker@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── dialog@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── dialog@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── dock@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── dock@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── drawer@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── drawer@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── dropdown-menu@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── dropdown-menu@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── editor@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── editor@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── effects@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── effects@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── empty@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── empty@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── field@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── field@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── files@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── files@hoff@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── git@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── git@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── graph@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── graph@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── hover-card@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── hover-card@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── icon-button@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── icon-button@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── icons@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── icons@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── input-group@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── input-group@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── input-otp@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── input-otp@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── input@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── input@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── item@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── item@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── kbd@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── kbd@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── label@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── label@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── marker@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── marker@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── menubar@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── menubar@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── message-scroller@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── message-scroller@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── message@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── message@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── navigation-menu@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── navigation-menu@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── pagination@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── pagination@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── panel-header@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── panel-header@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── panel@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── panel@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── popover@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── popover@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── progress@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── progress@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── questionnaire@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── questionnaire@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── radio-group@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── radio-group@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── resizable@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── resizable@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── scroll-area@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── scroll-area@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── select@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── select@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── separator@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── separator@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── sheet@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── sheet@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── sidebar@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── sidebar@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── silences-review-by-the-claude-fixture-adapted@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-claude-fixture-too-much-in-portuguese@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-claude-fixture@core@1200x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-claude-fixture@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-claude-fixture@hoff@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules-a-sequence-repeated@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules-can-t-compare@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules-larger-text-and-more-contrast@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules-the-sequence-as-one-action@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules-too-much-information@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules@core@1200x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences-review-by-the-rules@hoff@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── silences@hoff@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── skeleton@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── skeleton@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── slider@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── slider@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── spinner@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── spinner@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── stat@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── stat@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── switch@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── switch@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── table@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── table@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── tabs@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── tabs@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── textarea@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── textarea@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── theme@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── theme@hoff@1280x800/
│   │   │       │   └── scene.json
│   │   │       ├── toast@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── toast@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── todo@core@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── todo@core@390x844/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── todo@hoff@1280x800/
│   │   │       │   ├── frame.png
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── toggle-group@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── toggle-group@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── toggle@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── toggle@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── tooltip@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── tooltip@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── tree@core@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── tree@hoff@1280x800/
│   │   │       │   ├── scene.json
│   │   │       │   └── semantics.json
│   │   │       ├── typography@core@1280x800/
│   │   │       │   └── scene.json
│   │   │       └── typography@hoff@1280x800/
│   │   │           └── scene.json
│   │   ├── src/
│   │   │   ├── app/
│   │   │   │   ├── events/
│   │   │   │   │   └── ime.rs
│   │   │   │   ├── links/
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── pace/
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── access.rs
│   │   │   │   ├── background.rs
│   │   │   │   ├── capture.rs
│   │   │   │   ├── detached.rs
│   │   │   │   ├── entry.rs
│   │   │   │   ├── events.rs
│   │   │   │   ├── headless.rs
│   │   │   │   ├── links.rs
│   │   │   │   ├── menu.rs
│   │   │   │   ├── mirror.rs
│   │   │   │   ├── notify.rs
│   │   │   │   ├── pace.rs
│   │   │   │   ├── print.rs
│   │   │   │   ├── record.rs
│   │   │   │   ├── sleep.rs
│   │   │   │   ├── source.rs
│   │   │   │   ├── touch.rs
│   │   │   │   └── tray.rs
│   │   │   ├── model/
│   │   │   │   ├── silences/
│   │   │   │   │   ├── parse.rs
│   │   │   │   │   ├── probe.stderr
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── dock.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── silences.rs
│   │   │   │   ├── tests_dock.rs
│   │   │   │   ├── tests_todo.rs
│   │   │   │   └── todo.rs
│   │   │   ├── view/
│   │   │   │   ├── apps/
│   │   │   │   │   ├── app/
│   │   │   │   │   │   ├── semantics/
│   │   │   │   │   │   │   └── assistive.rs
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   └── semantics.rs
│   │   │   │   │   │   ├── draw.rs
│   │   │   │   │   │   ├── layout.rs
│   │   │   │   │   │   ├── semantics.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── capture/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── semantics.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── dock/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── files/
│   │   │   │   │   │   ├── semantics.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── git/
│   │   │   │   │   │   ├── desktop/
│   │   │   │   │   │   │   └── tests.rs
│   │   │   │   │   │   ├── desktop.rs
│   │   │   │   │   │   └── stub.rs
│   │   │   │   │   ├── silences/
│   │   │   │   │   │   ├── generated/
│   │   │   │   │   │   │   ├── access.rs
│   │   │   │   │   │   │   ├── offer.rs
│   │   │   │   │   │   │   └── page.rs
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── generated/
│   │   │   │   │   │   │   │   ├── access.rs
│   │   │   │   │   │   │   │   ├── change.rs
│   │   │   │   │   │   │   │   ├── checkpoint.rs
│   │   │   │   │   │   │   │   ├── contrast.rs
│   │   │   │   │   │   │   │   ├── demo.rs
│   │   │   │   │   │   │   │   └── transform.rs
│   │   │   │   │   │   │   ├── agent.rs
│   │   │   │   │   │   │   ├── composites.rs
│   │   │   │   │   │   │   ├── ffmpeg.rs
│   │   │   │   │   │   │   └── generated.rs
│   │   │   │   │   │   ├── draw.rs
│   │   │   │   │   │   ├── generated.rs
│   │   │   │   │   │   ├── jobs.rs
│   │   │   │   │   │   ├── layout.rs
│   │   │   │   │   │   ├── semantics.rs
│   │   │   │   │   │   ├── tests.rs
│   │   │   │   │   │   └── worker.rs
│   │   │   │   │   ├── surface_page/
│   │   │   │   │   │   ├── proposing/
│   │   │   │   │   │   │   ├── changing.rs
│   │   │   │   │   │   │   └── focus.rs
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── agent.rs
│   │   │   │   │   │   │   ├── claude.rs
│   │   │   │   │   │   │   ├── forms.rs
│   │   │   │   │   │   │   ├── items.rs
│   │   │   │   │   │   │   ├── measure.rs
│   │   │   │   │   │   │   └── proposals.rs
│   │   │   │   │   │   ├── asking.rs
│   │   │   │   │   │   ├── page.rs
│   │   │   │   │   │   ├── proposing.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── app.rs
│   │   │   │   │   ├── builder.rs
│   │   │   │   │   ├── dock.rs
│   │   │   │   │   ├── editor.rs
│   │   │   │   │   ├── files.rs
│   │   │   │   │   ├── git.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── silences.rs
│   │   │   │   │   └── surface_page.rs
│   │   │   │   ├── components/
│   │   │   │   │   ├── chart/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── avatar.rs
│   │   │   │   │   ├── badge.rs
│   │   │   │   │   ├── breadcrumb.rs
│   │   │   │   │   ├── button.rs
│   │   │   │   │   ├── card.rs
│   │   │   │   │   ├── chart.rs
│   │   │   │   │   ├── checkbox.rs
│   │   │   │   │   ├── chip.rs
│   │   │   │   │   ├── code_block.rs
│   │   │   │   │   ├── context_menu.rs
│   │   │   │   │   ├── dialog.rs
│   │   │   │   │   ├── empty.rs
│   │   │   │   │   ├── graph.rs
│   │   │   │   │   ├── icon_button.rs
│   │   │   │   │   ├── input.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── navigation_menu.rs
│   │   │   │   │   ├── panel.rs
│   │   │   │   │   ├── panel_header.rs
│   │   │   │   │   ├── progress.rs
│   │   │   │   │   ├── resizable.rs
│   │   │   │   │   ├── scroll_area.rs
│   │   │   │   │   ├── select.rs
│   │   │   │   │   ├── separator.rs
│   │   │   │   │   ├── sidebar.rs
│   │   │   │   │   ├── skeleton.rs
│   │   │   │   │   ├── slider.rs
│   │   │   │   │   ├── spinner.rs
│   │   │   │   │   ├── stat.rs
│   │   │   │   │   ├── switch.rs
│   │   │   │   │   ├── table.rs
│   │   │   │   │   ├── tabs.rs
│   │   │   │   │   ├── toast.rs
│   │   │   │   │   ├── tooltip.rs
│   │   │   │   │   └── tree.rs
│   │   │   │   ├── dispatch/
│   │   │   │   │   ├── component/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── keyboard/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── overlay/
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── shell/
│   │   │   │   │   │   ├── print.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── agent.rs
│   │   │   │   │   ├── component.rs
│   │   │   │   │   ├── detached.rs
│   │   │   │   │   ├── keyboard.rs
│   │   │   │   │   ├── overlay.rs
│   │   │   │   │   ├── reader.rs
│   │   │   │   │   └── shell.rs
│   │   │   │   ├── foundations/
│   │   │   │   │   ├── effects.rs
│   │   │   │   │   ├── icons.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── theme.rs
│   │   │   │   │   └── typography.rs
│   │   │   │   ├── pages/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── table.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── clipboard.rs
│   │   │   │   ├── dispatch.rs
│   │   │   │   ├── draw.rs
│   │   │   │   ├── field_menu.rs
│   │   │   │   ├── ime.rs
│   │   │   │   ├── input.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── panes.rs
│   │   │   │   ├── render.rs
│   │   │   │   └── tests.rs
│   │   │   ├── app.rs
│   │   │   ├── keys.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   ├── main_web.rs
│   │   │   └── renderer.rs
│   │   ├── tests/
│   │   │   ├── component_pages/
│   │   │   │   ├── buttons.rs
│   │   │   │   ├── choices.rs
│   │   │   │   ├── clipboard.rs
│   │   │   │   ├── field_menu.rs
│   │   │   │   ├── harness.rs
│   │   │   │   ├── ime.rs
│   │   │   │   ├── keyboard.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── promises.rs
│   │   │   │   └── values.rs
│   │   │   ├── login/
│   │   │   │   └── mod.rs
│   │   │   ├── access_tree.rs
│   │   │   ├── capture.rs
│   │   │   ├── composites.rs
│   │   │   ├── demo.rs
│   │   │   ├── detached.rs
│   │   │   ├── edit_literal.rs
│   │   │   ├── ephemeral.rs
│   │   │   ├── links.rs
│   │   │   ├── mirror.rs
│   │   │   ├── mirror_input.rs
│   │   │   ├── mirror_mask.rs
│   │   │   ├── mirror_probe.rs
│   │   │   ├── print.rs
│   │   │   ├── record.rs
│   │   │   ├── reduced_motion.rs
│   │   │   ├── sleep_wake.rs
│   │   │   ├── uir_fixture.rs
│   │   │   └── view_source.rs
│   │   └── Cargo.toml
│   ├── surface/
│   │   ├── src/
│   │   │   ├── host/
│   │   │   │   └── prefer.rs
│   │   │   ├── code.rs
│   │   │   ├── draw.rs
│   │   │   ├── expand.rs
│   │   │   ├── host.rs
│   │   │   ├── layout.rs
│   │   │   ├── lib.rs
│   │   │   ├── motion.rs
│   │   │   ├── project.rs
│   │   │   ├── surface.rs
│   │   │   ├── token.rs
│   │   │   └── widget.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   ├── mixer-left.ir.json
│   │   │   │   ├── mixer-right.ir.json
│   │   │   │   ├── showcase-silences-compare.ir.json
│   │   │   │   ├── showcase-silences-review.ir.json
│   │   │   │   └── showcase-todo-columns.ir.json
│   │   │   ├── support/
│   │   │   │   └── mod.rs
│   │   │   ├── adapt.rs
│   │   │   ├── checkpoint.rs
│   │   │   ├── code.rs
│   │   │   ├── emphasis.rs
│   │   │   ├── events.rs
│   │   │   ├── frames.rs
│   │   │   ├── materialize.rs
│   │   │   ├── measure.rs
│   │   │   ├── motion.rs
│   │   │   ├── policy.rs
│   │   │   ├── prefer.rs
│   │   │   ├── promote.rs
│   │   │   ├── render.rs
│   │   │   └── transform.rs
│   │   └── Cargo.toml
│   ├── svg/
│   │   ├── src/
│   │   │   ├── cnv.rs
│   │   │   ├── lib.rs
│   │   │   ├── tess.rs
│   │   │   └── tests.rs
│   │   └── Cargo.toml
│   ├── uir/
│   │   ├── fixtures/
│   │   │   ├── ide-commit-form.ir.json
│   │   │   ├── ide-review-commit.ir.json
│   │   │   ├── ide-workbench.snapshot.json
│   │   │   ├── showcase-files.snapshot.json
│   │   │   ├── showcase-silences.snapshot.json
│   │   │   ├── showcase-todo.ir.json
│   │   │   └── showcase-todo.snapshot.json
│   │   ├── schema/
│   │   │   └── plev-ir-1.json
│   │   ├── src/
│   │   │   ├── transform/
│   │   │   │   ├── look.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── prefer.rs
│   │   │   │   ├── shape.rs
│   │   │   │   ├── tree.rs
│   │   │   │   └── view.rs
│   │   │   ├── verify/
│   │   │   │   ├── access.rs
│   │   │   │   ├── bind.rs
│   │   │   │   └── mod.rs
│   │   │   ├── caps.rs
│   │   │   ├── decode.rs
│   │   │   ├── error.rs
│   │   │   ├── ir.rs
│   │   │   ├── item.rs
│   │   │   ├── lib.rs
│   │   │   ├── policy.rs
│   │   │   ├── schema.rs
│   │   │   ├── token.rs
│   │   │   └── vocab.rs
│   │   ├── tests/
│   │   │   ├── jsonschema/
│   │   │   │   └── mod.rs
│   │   │   ├── support/
│   │   │   │   └── mod.rs
│   │   │   ├── code.rs
│   │   │   ├── fixtures.rs
│   │   │   ├── identity.rs
│   │   │   ├── prefer.rs
│   │   │   ├── prop.rs
│   │   │   ├── rules.rs
│   │   │   └── transform.rs
│   │   └── Cargo.toml
│   ├── urnaui/
│   │   ├── src/
│   │   │   ├── model/
│   │   │   │   ├── backend.rs
│   │   │   │   ├── bench.rs
│   │   │   │   ├── embed.rs
│   │   │   │   ├── frames.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── recents.rs
│   │   │   │   ├── subprocess.rs
│   │   │   │   ├── types.rs
│   │   │   │   ├── urnaread.rs
│   │   │   │   ├── web.rs
│   │   │   │   └── worker.rs
│   │   │   ├── view/
│   │   │   │   ├── explorer/
│   │   │   │   │   └── dialogs.rs
│   │   │   │   ├── chunks.rs
│   │   │   │   ├── explorer.rs
│   │   │   │   ├── field.rs
│   │   │   │   ├── graph.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── open.rs
│   │   │   │   ├── overview.rs
│   │   │   │   ├── search.rs
│   │   │   │   └── stats.rs
│   │   │   ├── app.rs
│   │   │   ├── keys.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   ├── renderer.rs
│   │   │   └── web_picker.rs
│   │   ├── tests/
│   │   │   ├── backend_integration.rs
│   │   │   ├── media_corpus.rs
│   │   │   └── web_reader_parity.rs
│   │   ├── Cargo.lock
│   │   └── Cargo.toml
│   └── wire/
│       ├── src/
│       │   ├── bin/
│       │   │   └── plev.rs
│       │   ├── b64.rs
│       │   ├── client.rs
│       │   ├── codec.rs
│       │   ├── error.rs
│       │   ├── lib.rs
│       │   ├── place.rs
│       │   ├── port.rs
│       │   ├── proto.rs
│       │   ├── ring.rs
│       │   ├── scene.rs
│       │   ├── scene_read.rs
│       │   ├── server.rs
│       │   ├── shown.rs
│       │   ├── sink.rs
│       │   └── traffic.rs
│       ├── tests/
│       │   ├── toy/
│       │   │   └── mod.rs
│       │   ├── cli.rs
│       │   ├── codec.rs
│       │   ├── env.rs
│       │   ├── look.rs
│       │   ├── ring.rs
│       │   ├── scene.rs
│       │   ├── server.rs
│       │   ├── shown.rs
│       │   ├── socket.rs
│       │   └── traffic.rs
│       └── Cargo.toml
├── docs/
│   ├── ADR/
│   │   ├── a-covered-window-does-not-draw-and-wakes-new.md
│   │   ├── a-delta-carries-the-node-a-refusal-names-who-acted.md
│   │   ├── a-link-is-not-a-person.md
│   │   ├── a-renderer-upgrade-is-proven-by-a-full-capture.md
│   │   ├── a-terminal-stage-carries-the-revision.md
│   │   ├── a-tick-asks-for-a-deadline.md
│   │   ├── accessibility-is-production-ready-per-platform.md
│   │   ├── accessibility-projects-the-semantic-snapshot.md
│   │   ├── actions-carry-observed-state.md
│   │   ├── adaptive-transforms-preserve-identity.md
│   │   ├── adr-003-srp-modularization.md
│   │   ├── adr-004-hidpi-projection.md
│   │   ├── agents-act-through-the-semantic-contract.md
│   │   ├── ai-native-single-process.md
│   │   ├── an-act-rests-on-what-its-actor-was-shown.md
│   │   ├── android-emulator-deploy.md
│   │   ├── animation-pattern-lerp.md
│   │   ├── arxiv-paper-draft.md
│   │   ├── arxiv-paper-outline.md
│   │   ├── async-gpu-init-and-single-wasm-entry.md
│   │   ├── benchmark-results.md
│   │   ├── binary-animation-format-with-discovered-deltas.md
│   │   ├── brief-strengths.md
│   │   ├── capture-is-a-job-the-person-turns-on.md
│   │   ├── clippy-zero-warnings.md
│   │   ├── commit-checks-the-index-it-observed.md
│   │   ├── component-design.md
│   │   ├── comps-one-design-system-crate.md
│   │   ├── content-driven-layout-not-fixed-constants.md
│   │   ├── core-theme-tailwind-classes-as-recipes.md
│   │   ├── design-for-ranges-not-screens-and-devices.md
│   │   ├── dsl-narrate-design.md
│   │   ├── effects-architecture.md
│   │   ├── embed-an-app-view-by-frame-origin.md
│   │   ├── embed-every-font-weight-in-use.md
│   │   ├── embedded-fonts-by-script.md
│   │   ├── error-handling-lint-cleanup.md
│   │   ├── extracted-patterns.md
│   │   ├── file-dialogs-and-drops-are-capabilities.md
│   │   ├── generative-runtime-results.md
│   │   ├── git-backend-gix-reads-cli-mutations.md
│   │   ├── glyph-raster-identity-and-atlas-isolation.md
│   │   ├── gpu-device-loss-recovers-in-place.md
│   │   ├── headless-render-shares-the-frame-path.md
│   │   ├── hot-reload-design.md
│   │   ├── hot-reload-on-by-default-in-gpu-context.md
│   │   ├── import-foreign-formats-by-conversion-not-embedding.md
│   │   ├── importers-target-plev-ir.md
│   │   ├── input-system-design.md
│   │   ├── ios-build.md
│   │   ├── layer-system.md
│   │   ├── layout-engine.md
│   │   ├── lean-semantic-form-and-one-redaction.md
│   │   ├── linearize-colors-before-the-gpu.md
│   │   ├── makepad-gap-analysis.md
│   │   ├── materialized-surfaces-from-verified-ir.md
│   │   ├── measured-design-tokens-over-eyeballed-values.md
│   │   ├── mobile-specifics.md
│   │   ├── monster-format-v0.md
│   │   ├── motion-trails-by-position-history.md
│   │   ├── narrate-hot-reload-adr.md
│   │   ├── native-dialogs-answer-the-same-contract.md
│   │   ├── native-menus-are-another-view-of-capabilities.md
│   │   ├── official-app-pattern.md
│   │   ├── one-knowledge-tree-and-a-minimal-root.md
│   │   ├── one-text-style-for-measurement-and-drawing.md
│   │   ├── parley-vs-cosmic-text.md
│   │   ├── plev-ir-typed-and-verified.md
│   │   ├── plev-vs-makepad-report.md
│   │   ├── printing-is-the-frame-on-paper.md
│   │   ├── proposers-propose-the-runtime-verifies.md
│   │   ├── README.md
│   │   ├── regression-by-meaning-before-pixels.md
│   │   ├── render-into-an-srgb-view-format.md
│   │   ├── render-on-demand-requires-explicit-invalidation.md
│   │   ├── responsiveness-multiplatform-and-fidelity.md
│   │   ├── scene-travels-as-data.md
│   │   ├── semantic-binding-beside-the-scene.md
│   │   ├── semantic-contract-as-pure-projection.md
│   │   ├── session-recording-replay-not-reexecution.md
│   │   ├── sessions-become-regression-tests.md
│   │   ├── shader-hot-reload-adr.md
│   │   ├── showcase-one-page-per-component.md
│   │   ├── signal-system-design.md
│   │   ├── srp-modularization.md
│   │   ├── svg-export-reads-the-scene.md
│   │   ├── text-editing-core-follows-helix.md
│   │   ├── text-moves-by-screen-order-and-shapes-by-line.md
│   │   ├── the-inspector-reads-a-session-never-runs-it.md
│   │   ├── the-keyboard-walks-the-projection.md
│   │   ├── the-persons-text-and-clipboard-stay-the-persons.md
│   │   ├── the-silences-app-is-the-fixed-screen.md
│   │   ├── touch-as-synthesized-pointer-events.md
│   │   ├── touch-gesture-design.md
│   │   ├── transpiler-reports-every-unmapped-construct.md
│   │   ├── tray-and-notifications-are-capabilities-out-of-the-window.md
│   │   ├── ui-asset-compiler.md
│   │   ├── urnaui-media-frames-in-place-via-ffmpeg-subfile.md
│   │   ├── view-source-for-pixels.md
│   │   ├── view-trait-design.md
│   │   ├── wasm-plugin-architecture.md
│   │   ├── wasm-webgpu-validation.md
│   │   ├── what-identical-means.md
│   │   ├── windows-are-hosts-the-app-owns-the-state.md
│   │   ├── worker-results-wake-the-event-loop.md
│   │   └── workspace-engine-at-root-libs-in-crates-demos-in-examples.md
│   ├── arc/
│   │   └── arc.toml
│   ├── how-to/
│   │   ├── build-and-serve-the-web-target.md
│   │   ├── code-against-the-plev-engine.md
│   │   ├── orchestrate-coding-agents-on-this-repo.md
│   │   └── validate-visuals-by-pixel.md
│   ├── mission/
│   │   ├── state/
│   │   │   ├── journal.md
│   │   │   ├── learnings.md
│   │   │   └── truth.md
│   │   ├── steps/
│   │   │   ├── checked/
│   │   │   │   ├── changelogs/
│   │   │   │   │   ├── gap-1.md
│   │   │   │   │   ├── task-01.md
│   │   │   │   │   ├── task-02.md
│   │   │   │   │   ├── task-03.md
│   │   │   │   │   ├── task-04.md
│   │   │   │   │   ├── task-05.md
│   │   │   │   │   ├── task-06.md
│   │   │   │   │   ├── task-07.md
│   │   │   │   │   ├── task-08.md
│   │   │   │   │   ├── task-09.md
│   │   │   │   │   ├── task-10.md
│   │   │   │   │   ├── task-11.md
│   │   │   │   │   ├── task-12.md
│   │   │   │   │   ├── task-13.md
│   │   │   │   │   ├── task-14.md
│   │   │   │   │   ├── task-15.md
│   │   │   │   │   ├── task-16.md
│   │   │   │   │   ├── task-17.md
│   │   │   │   │   ├── task-18.md
│   │   │   │   │   ├── task-19.md
│   │   │   │   │   ├── task-20.md
│   │   │   │   │   ├── task-21.md
│   │   │   │   │   ├── task-22.md
│   │   │   │   │   ├── task-23.md
│   │   │   │   │   ├── task-24.md
│   │   │   │   │   ├── task-25.md
│   │   │   │   │   ├── task-26.md
│   │   │   │   │   ├── task-27.md
│   │   │   │   │   ├── task-28.md
│   │   │   │   │   ├── task-29.md
│   │   │   │   │   ├── task-30.md
│   │   │   │   │   ├── task-31.md
│   │   │   │   │   ├── task-32.md
│   │   │   │   │   ├── task-33.md
│   │   │   │   │   ├── task-34.md
│   │   │   │   │   ├── task-35.md
│   │   │   │   │   ├── task-36.md
│   │   │   │   │   ├── task-37.md
│   │   │   │   │   ├── task-38.md
│   │   │   │   │   ├── task-42.md
│   │   │   │   │   ├── task-43.md
│   │   │   │   │   ├── task-44.md
│   │   │   │   │   ├── task-45.md
│   │   │   │   │   ├── task-46.md
│   │   │   │   │   ├── task-47.md
│   │   │   │   │   ├── task-48.md
│   │   │   │   │   ├── task-49.md
│   │   │   │   │   ├── task-50.md
│   │   │   │   │   ├── task-51.md
│   │   │   │   │   ├── task-52.md
│   │   │   │   │   └── task-ref.md
│   │   │   │   ├── checkpoint-b/
│   │   │   │   │   ├── 02-fixed-detected.png
│   │   │   │   │   ├── 03-generated-rules.png
│   │   │   │   │   ├── 04-generated-claude.png
│   │   │   │   │   ├── 05-after-task.png
│   │   │   │   │   ├── r2-02-fixed-detected.png
│   │   │   │   │   ├── r2-03-generated-rules.png
│   │   │   │   │   ├── r2-04-generated-claude.png
│   │   │   │   │   └── r2-05-after-task.png
│   │   │   │   ├── sessao-comps-learnings.md
│   │   │   │   ├── sessao-core-buttons-learnings.md
│   │   │   │   ├── sessao-generative-runtime-learnings.md
│   │   │   │   ├── sessao-hot-reload-learnings.md
│   │   │   │   ├── sessao-showcase-pages-learnings.md
│   │   │   │   ├── task-01-done.md
│   │   │   │   ├── task-02-done.md
│   │   │   │   ├── task-03-done.md
│   │   │   │   ├── task-04-done.md
│   │   │   │   ├── task-05-done.md
│   │   │   │   ├── task-06-done.md
│   │   │   │   ├── task-07-done.md
│   │   │   │   ├── task-08-done.md
│   │   │   │   ├── task-09-done.md
│   │   │   │   ├── task-10-done.md
│   │   │   │   ├── task-11-done.md
│   │   │   │   ├── task-12-done.md
│   │   │   │   ├── task-13-done.md
│   │   │   │   ├── task-14-done.md
│   │   │   │   ├── task-15-done.md
│   │   │   │   ├── task-16-done.md
│   │   │   │   ├── task-17-done.md
│   │   │   │   ├── task-18-android-device-test-done.md
│   │   │   │   ├── task-19-ios-simulator-test-done.md
│   │   │   │   ├── task-20-wasm-visual-validation-done.md
│   │   │   │   ├── task-21-done.md
│   │   │   │   ├── task-22-done.md
│   │   │   │   ├── task-23-done.md
│   │   │   │   ├── task-24-done.md
│   │   │   │   ├── task-25-benchmarks-done.md
│   │   │   │   ├── task-26-arxiv-paper-done.md
│   │   │   │   ├── task-27-done.md
│   │   │   │   ├── task-28-done.md
│   │   │   │   ├── task-29-done.md
│   │   │   │   ├── task-30-accessibility-done.md
│   │   │   │   ├── task-31-vector-paths-done.md
│   │   │   │   ├── task-32-text-upgrade-assessment-done.md
│   │   │   │   ├── task-33-wasm-plugins-done.md
│   │   │   │   ├── task-34-exploration-extraction-done.md
│   │   │   │   ├── task-35-spring-analytical-solver-done.md
│   │   │   │   ├── task-36-signal-hardening-done.md
│   │   │   │   ├── task-37-animation-enhancements-done.md
│   │   │   │   ├── task-38-event-batching-done.md
│   │   │   │   ├── task-39-scene-cache-memoization-superseded.md
│   │   │   │   ├── task-42-plev-event-dispatch-superseded.md
│   │   │   │   ├── task-43-monster-codec-done.md
│   │   │   │   ├── task-44-lot-lottie-importer-done.md
│   │   │   │   ├── task-45-parser-transpiler-done.md
│   │   │   │   ├── task-46-rope-editing-core-done.md
│   │   │   │   ├── task-47-git-backend-done.md
│   │   │   │   ├── task-48-ide-git-client-done.md
│   │   │   │   ├── task-49-prime-creatures-done.md
│   │   │   │   ├── task-50-showcase-multiplatform-done.md
│   │   │   │   ├── task-51-workspace-restructure-done.md
│   │   │   │   ├── task-52-srp-modularization-done.md
│   │   │   │   ├── task-53-editor-and-git-in-the-showcase-done.md
│   │   │   │   ├── task-54-core-theme-buttons-poc-done.md
│   │   │   │   ├── task-55-showcase-one-page-per-component-done.md
│   │   │   │   ├── task-56-hot-reload-done.md
│   │   │   │   ├── task-57-semantic-contract-done.md
│   │   │   │   ├── task-57b-lean-wire-form-and-one-redaction-done.md
│   │   │   │   ├── task-58-actions-preconditions-receipts-done.md
│   │   │   │   ├── task-58b-decline-a-confirmation-done.md
│   │   │   │   ├── task-59-semantic-binding-and-one-path-done.md
│   │   │   │   ├── task-60-accessibility-projection-done.md
│   │   │   │   ├── task-61-agent-surface-done.md
│   │   │   │   ├── task-62-ide-human-agent-proof-done.md
│   │   │   │   ├── task-63-flight-recorder-done.md
│   │   │   │   ├── task-64-session-inspector-and-tests-done.md
│   │   │   │   ├── task-65-pixel-to-source-done.md
│   │   │   │   ├── task-66-headless-first-class-done.md
│   │   │   │   ├── task-66b-svg-export-done.md
│   │   │   │   ├── task-67-structural-regression-done.md
│   │   │   │   ├── task-68-plev-ir-and-verifier-done.md
│   │   │   │   ├── task-69-ephemeral-generative-surfaces-done.md
│   │   │   │   ├── task-70-adaptive-transformations-done.md
│   │   │   │   ├── task-71-compiler-target-done.md
│   │   │   │   ├── task-72-ui-asset-compiler-done.md
│   │   │   │   ├── task-76-platform-integration-done.md
│   │   │   │   ├── task-77-visual-determinism-done.md
│   │   │   │   ├── task-79-scene-over-socket-done.md
│   │   │   │   ├── task-81-select-scrolling-panel-done.md
│   │   │   │   ├── task-82-caret-blink-render-on-demand-done.md
│   │   │   │   ├── task-83-delta-carries-node-refusal-names-actor-done.md
│   │   │   │   ├── task-84-ide-refresh-never-goes-back-done.md
│   │   │   │   ├── task-85-the-commit-a-receipt-announces-is-projected-done.md
│   │   │   │   ├── task-86-the-ide-opens-the-agent-port-done.md
│   │   │   │   ├── task-87-a-terminal-stage-carries-the-revision-done.md
│   │   │   │   ├── task-88-the-lab-measures-what-the-agent-rested-on-done.md
│   │   │   │   ├── task-89-an-act-rests-only-on-what-was-shown-done.md
│   │   │   │   ├── task-90-a-refusal-names-who-acted-since-the-look-done.md
│   │   │   │   ├── task-91-the-dispatcher-knows-when-the-agent-looked-done.md
│   │   │   │   ├── task-92-the-agent-asks-the-person-done.md
│   │   │   │   ├── task-93-the-lab-person-answers-done.md
│   │   │   │   ├── task-94-the-refusal-points-at-the-question-done.md
│   │   │   │   ├── task-95-the-todo-projects-its-items-as-a-list-done.md
│   │   │   │   ├── task-96-ide-generated-surface-done.md
│   │   │   │   ├── task-97-code-and-diff-in-the-ir-done.md
│   │   │   │   └── task-ref-research-briefing-done.md
│   │   │   ├── ongoing/
│   │   │   │   ├── task-73-incremental-dependency-system.md
│   │   │   │   ├── task-74-text-and-editing.md
│   │   │   │   ├── task-75-platform-accessibility.md
│   │   │   │   ├── task-78-ai-native-apps.md
│   │   │   │   └── task-80-proof-qualitatively-different-software.md
│   │   │   └── pending/
│   │   │       └── task-41-license-and-public-release.md
│   │   ├── autonomy.md
│   │   ├── generative-runtime-guidelines.md
│   │   ├── generative-runtime-program.md
│   │   ├── generative-runtime-thesis.md
│   │   ├── readme.md
│   │   └── task-73-measurements.md
│   ├── refs/
│   │   ├── accessibility.md
│   │   ├── animation-motion.md
│   │   ├── charts-visualization.md
│   │   ├── competitive-positioning.md
│   │   ├── competitors-by-tier.md
│   │   ├── competitors.md
│   │   ├── emulators-wasm-runtimes.md
│   │   ├── extraction-fase-ef.md
│   │   ├── integration-candidates.md
│   │   ├── linebender-ecosystem.md
│   │   ├── math-physics-geometry.md
│   │   ├── mon-experiment-notes.md
│   │   ├── pattern-extraction-parley-lyon-glam.md
│   │   ├── pattern-extraction-tui-apps.md
│   │   ├── technology-radar.md
│   │   ├── tui-patterns.md
│   │   └── wasm-tooling.md
│   ├── catalog.md
│   ├── CHANGELOG.md
│   └── LICENSE
├── ios/
│   ├── showcase/
│   │   ├── Sources/
│   │   │   └── main.m
│   │   ├── build_ios.sh
│   │   ├── project.yml
│   │   └── run_ios.sh
│   └── Info.plist
├── refs/
│   └── lottie/
│       ├── cards/
│       │   ├── motions/
│       │   │   ├── bar/
│       │   │   │   ├── 01.json
│       │   │   │   └── 01.lottie
│       │   │   ├── listanimations.json
│       │   │   └── listanimations.lottie
│       │   ├── 01.webm
│       │   ├── cards.svg
│       │   ├── json.json
│       │   └── listanimation.webm
│       ├── explosion/
│       │   ├──  Explosion.webm
│       │   ├── Explosion.json
│       │   ├── Explosion_dot.lottie
│       │   ├── Fire Explosion.lottie
│       │   ├── json_optimized_Explosion.json
│       │   └── optimized_Explosion.lottie
│       ├── girl/
│       │   ├── cards.lottie
│       │   ├── dot.lottie
│       │   ├── girl.webm
│       │   ├── json.json
│       │   ├── optmized.json
│       │   └── optmized.lottie
│       ├── MONEY/
│       │   ├── interactivebototnbar/
│       │   │   ├── interativebtn/
│       │   │   │   ├── btnexpand/
│       │   │   │   │   ├── addbtn/
│       │   │   │   │   │   ├── btnexpand/
│       │   │   │   │   │   │   ├── catsmile/
│       │   │   │   │   │   │   │   ├── 18e51725-5d27-49f6-aac9-278dff2870da.json
│       │   │   │   │   │   │   │   ├── 18e51725-5d27-49f6-aac9-278dff2870da.lottie
│       │   │   │   │   │   │   │   └── 230x499bb.webp
│       │   │   │   │   │   │   ├── 3f946e2a-d04a-45d4-a4ab-97dcac5a732a.json
│       │   │   │   │   │   │   ├── 3f946e2a-d04a-45d4-a4ab-97dcac5a732a.lottie
│       │   │   │   │   │   │   └── 3f946e2a-d04a-45d4-a4ab-97dcac5a732a.svg
│       │   │   │   │   │   ├── 282ada5d-a74b-4ece-9e60-0118f22e7b96.json
│       │   │   │   │   │   ├── 282ada5d-a74b-4ece-9e60-0118f22e7b96.lottie
│       │   │   │   │   │   └── 282ada5d-a74b-4ece-9e60-0118f22e7b96.svg
│       │   │   │   │   ├── a4839e1d-2947-4e02-b9b7-2b94bf1409c0.json
│       │   │   │   │   ├── a4839e1d-2947-4e02-b9b7-2b94bf1409c0.lottie
│       │   │   │   │   └── a4839e1d-2947-4e02-b9b7-2b94bf1409c0.svg
│       │   │   │   ├── 53fdc2f8-6c41-465c-92e7-ab540d2c90f6.json
│       │   │   │   ├── 53fdc2f8-6c41-465c-92e7-ab540d2c90f6.lottie
│       │   │   │   └── 53fdc2f8-6c41-465c-92e7-ab540d2c90f6.svg
│       │   │   ├── 0b87c13f-61d1-4560-a99b-10c571634a1e.json
│       │   │   ├── 0b87c13f-61d1-4560-a99b-10c571634a1e.lottie
│       │   │   └── 0b87c13f-61d1-4560-a99b-10c571634a1e.svg
│       │   ├── 5674eb5c-1187-11ee-a031-df397cf00d68.json
│       │   ├── 5674eb5c-1187-11ee-a031-df397cf00d68.lottie
│       │   └── 5674eb5c-1187-11ee-a031-df397cf00d68.svg
│       └── SNAKE/
│           ├── fd5e87b4-1189-11ee-9745-e700d1385b38.gif
│           ├── fd5e87b4-1189-11ee-9745-e700d1385b38.json
│           └── fd5e87b4-1189-11ee-9745-e700d1385b38.lottie
├── script/
│   ├── capture-android
│   ├── capture-common.sh
│   ├── capture-ios
│   ├── capture-web
│   ├── figma-fixture
│   ├── font-subset
│   ├── gate
│   ├── web
│   └── web-urnaui
├── web/
│   ├── dist/
│   │   └── .stage/
│   ├── urnaui/
│   │   ├── index.html
│   │   └── Trunk.toml
│   ├── index.html
│   └── Trunk.toml
├── .editorconfig
├── .gitattributes
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
├── rust-toolchain.toml
└── rustfmt.toml
```

487 pastas, 1904 arquivos.
