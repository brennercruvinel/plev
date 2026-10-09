# boiler, árvore completa

❯ Bom dia, iremos dar prosseguimento da nossa tarefa, onde já está muito

  bem detalhada e descrito do que tem que ser feito. Mover, renomear e

  organizar a arvore atual do plev a nova estrutura listada abaixo:  

```text
plevdev/
├── .cargo/
│   └── config.toml
├── .design/
│   └── .gitkeep
├── .devops/
│   ├── agent/
│   │   └── skill/
│   │       └── .gitkeep
│   ├── check/
│   │   ├── lychee.toml
│   │   ├── nextest.toml
│   │   └── typos.toml
│   └── rules/
│       ├── mantra/
│       │   └── .gitkeep
│       └── taboos/
│           └── .gitkeep
├── .github/
│   ├── workflows/
│   │   └── gatecheck.yml
│   └── CODEOWNERS
├── .secops/
│   └── .gitkeep
├── apps/
│   ├── agenteval/
│   │   ├── src/
│   │   │   ├── arms/
│   │   │   │   ├── git.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── plev.rs
│   │   │   │   ├── review.rs
│   │   │   │   ├── screenshot.rs
│   │   │   │   └── scripted.rs
│   │   │   ├── answer.rs
│   │   │   ├── args.rs
│   │   │   ├── asked.rs
│   │   │   ├── finish.rs
│   │   │   ├── host.rs
│   │   │   ├── main.rs
│   │   │   ├── refused.rs
│   │   │   ├── report.rs
│   │   │   ├── story.rs
│   │   │   └── subject.rs
│   │   ├── tests/
│   │   │   ├── experiment.rs
│   │   │   └── review.rs
│   │   └── Cargo.toml
│   ├── idestudio/
│   │   ├── src/
│   │   │   ├── capability/
│   │   │   │   ├── agent.rs
│   │   │   │   ├── execute.rs
│   │   │   │   ├── ids.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── notice.rs
│   │   │   │   ├── person.rs
│   │   │   │   ├── project.rs
│   │   │   │   ├── tests.rs
│   │   │   │   └── truth.rs
│   │   │   ├── generative/
│   │   │   │   ├── asking.rs
│   │   │   │   ├── claude.rs
│   │   │   │   ├── column.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── row.rs
│   │   │   │   └── scope.rs
│   │   │   ├── interfaces/
│   │   │   │   ├── panels/
│   │   │   │   │   ├── layout/
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   ├── overlays.rs
│   │   │   │   │   │   │   ├── panels.rs
│   │   │   │   │   │   │   ├── scrolling.rs
│   │   │   │   │   │   │   └── wake.rs
│   │   │   │   │   │   ├── input.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── overlays.rs
│   │   │   │   │   │   └── render.rs
│   │   │   │   │   ├── commit_form.rs
│   │   │   │   │   ├── diff.rs
│   │   │   │   │   ├── header.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── sidebar.rs
│   │   │   │   │   ├── stacks.rs
│   │   │   │   │   └── unassigned.rs
│   │   │   │   ├── window/
│   │   │   │   │   ├── record/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── shell.rs
│   │   │   │   │   ├── keyboard.rs
│   │   │   │   │   ├── keymap.json
│   │   │   │   │   ├── keymap.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── pace.rs
│   │   │   │   │   └── renderer.rs
│   │   │   │   └── mod.rs
│   │   │   ├── repository/
│   │   │   │   ├── adapters.rs
│   │   │   │   ├── eval_repo.rs
│   │   │   │   ├── git.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── status.rs
│   │   │   │   └── watcher.rs
│   │   │   ├── lib.rs
│   │   │   └── main.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   └── sessions/
│   │   │   │       ├── a2ppa.marks
│   │   │   │       ├── a2ppa.monster
│   │   │   │       ├── bug84.marks
│   │   │   │       └── bug84.monster
│   │   │   ├── rerun/
│   │   │   │   ├── eval.rs
│   │   │   │   ├── harness.rs
│   │   │   │   ├── inflight.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── marks.rs
│   │   │   │   ├── report.rs
│   │   │   │   └── toy.rs
│   │   │   ├── support/
│   │   │   │   └── mod.rs
│   │   │   ├── agent_port.rs
│   │   │   ├── claude.rs
│   │   │   ├── human_agent.rs
│   │   │   ├── line.rs
│   │   │   ├── native_confirm.rs
│   │   │   ├── observe_cost.rs
│   │   │   ├── plevir_fixture.rs
│   │   │   ├── record.rs
│   │   │   ├── review.rs
│   │   │   ├── review_acts.rs
│   │   │   └── surface.rs
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
│   │   │       ├── privacy.rs
│   │   │       ├── readonly.rs
│   │   │       ├── state.rs
│   │   │       ├── support.rs
│   │   │       └── viewport.rs
│   │   └── Cargo.toml
│   ├── mirroring/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   └── show.rs
│   │   └── Cargo.toml
│   ├── uigallery/
│   │   ├── android/
│   │   │   ├── app/
│   │   │   │   ├── src/
│   │   │   │   │   └── main/
│   │   │   │   │       ├── kotlin/
│   │   │   │   │       │   └── com/
│   │   │   │   │       │       └── plev/
│   │   │   │   │       │           └── uigallery/
│   │   │   │   │       │               └── MainActivity.kt
│   │   │   │   │       ├── res/
│   │   │   │   │       │   └── values/
│   │   │   │   │       │       └── themes.xml
│   │   │   │   │       └── AndroidManifest.xml
│   │   │   │   └── build.gradle.kts
│   │   │   ├── gradle/
│   │   │   │   ├── wrapper/
│   │   │   │   │   ├── gradle-wrapper.jar
│   │   │   │   │   └── gradle-wrapper.properties
│   │   │   │   └── libs.versions.toml
│   │   │   ├── build.gradle.kts
│   │   │   ├── gradle.properties
│   │   │   ├── gradlew
│   │   │   ├── gradlew.bat
│   │   │   └── settings.gradle.kts
│   │   ├── examples/
│   │   │   └── videos/
│   │   │       ├── agent.rs
│   │   │       ├── main.rs
│   │   │       └── shell.rs
│   │   ├── ios/
│   │   │   ├── Sources/
│   │   │   │   └── main.m
│   │   │   ├── Info.plist
│   │   │   └── project.yml
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   └── web.rs
│   │   │   ├── domain/
│   │   │   │   ├── silences/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── parse.rs
│   │   │   │   │   ├── probe.stderr
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── tests/
│   │   │   │   │   ├── dock.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── todo.rs
│   │   │   │   ├── dock.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── todo.rs
│   │   │   ├── shell/
│   │   │   │   ├── system/
│   │   │   │   │   ├── links/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── background.rs
│   │   │   │   │   ├── capture.rs
│   │   │   │   │   ├── menu.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── notify.rs
│   │   │   │   │   ├── print.rs
│   │   │   │   │   ├── sleep.rs
│   │   │   │   │   ├── touch.rs
│   │   │   │   │   └── tray.rs
│   │   │   │   ├── window/
│   │   │   │   │   ├── events/
│   │   │   │   │   │   ├── ime.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── pace/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── access.rs
│   │   │   │   │   ├── detached.rs
│   │   │   │   │   ├── entry.rs
│   │   │   │   │   ├── headless.rs
│   │   │   │   │   ├── mirror.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── record.rs
│   │   │   │   │   └── source.rs
│   │   │   │   └── mod.rs
│   │   │   ├── views/
│   │   │   │   ├── apps/
│   │   │   │   │   ├── capture/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── semantics.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── dock/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── files/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── semantics.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── git/
│   │   │   │   │   │   ├── desktop/
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   └── tests.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── stub.rs
│   │   │   │   │   ├── silences/
│   │   │   │   │   │   ├── fixed_page/
│   │   │   │   │   │   │   ├── draw.rs
│   │   │   │   │   │   │   ├── jobs.rs
│   │   │   │   │   │   │   ├── layout.rs
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   ├── semantics.rs
│   │   │   │   │   │   │   └── worker.rs
│   │   │   │   │   │   ├── generative/
│   │   │   │   │   │   │   ├── access.rs
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   ├── offer.rs
│   │   │   │   │   │   │   └── page.rs
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── generative/
│   │   │   │   │   │   │   │   ├── access.rs
│   │   │   │   │   │   │   │   ├── change.rs
│   │   │   │   │   │   │   │   ├── checkpoint.rs
│   │   │   │   │   │   │   │   ├── contrast.rs
│   │   │   │   │   │   │   │   ├── demo.rs
│   │   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   │   └── transform.rs
│   │   │   │   │   │   │   ├── agent.rs
│   │   │   │   │   │   │   ├── composites.rs
│   │   │   │   │   │   │   ├── ffmpeg.rs
│   │   │   │   │   │   │   └── mod.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── surface/
│   │   │   │   │   │   ├── offering/
│   │   │   │   │   │   │   ├── changing.rs
│   │   │   │   │   │   │   ├── focus.rs
│   │   │   │   │   │   │   └── mod.rs
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── agent.rs
│   │   │   │   │   │   │   ├── claude.rs
│   │   │   │   │   │   │   ├── forms.rs
│   │   │   │   │   │   │   ├── items.rs
│   │   │   │   │   │   │   ├── measure.rs
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   └── offers.rs
│   │   │   │   │   │   ├── asking.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── page.rs
│   │   │   │   │   ├── todo/
│   │   │   │   │   │   ├── semantics/
│   │   │   │   │   │   │   ├── assistive.rs
│   │   │   │   │   │   │   └── mod.rs
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   └── semantics.rs
│   │   │   │   │   │   ├── draw.rs
│   │   │   │   │   │   ├── layout.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── builder.rs
│   │   │   │   │   ├── editor.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── components/
│   │   │   │   │   ├── chart/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── avatar.rs
│   │   │   │   │   ├── badge.rs
│   │   │   │   │   ├── breadcrumb.rs
│   │   │   │   │   ├── button.rs
│   │   │   │   │   ├── card.rs
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
│   │   │   │   ├── compositor/
│   │   │   │   │   ├── draw.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── render.rs
│   │   │   │   ├── dispatcher/
│   │   │   │   │   ├── component/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── keyboard/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── overlay/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── shell/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── print.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── agent.rs
│   │   │   │   │   ├── detached.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── reader.rs
│   │   │   │   ├── foundations/
│   │   │   │   │   ├── effects.rs
│   │   │   │   │   ├── icons.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── theme.rs
│   │   │   │   │   └── typography.rs
│   │   │   │   ├── pages/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── panes.rs
│   │   │   │   │   ├── table.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── text_input/
│   │   │   │   │   ├── clipboard.rs
│   │   │   │   │   ├── field_menu.rs
│   │   │   │   │   ├── ime.rs
│   │   │   │   │   ├── input.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── tests.rs
│   │   │   ├── keyboard.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
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
│   │   │   ├── fixtures/
│   │   │   │   └── sessions/
│   │   │   │       ├── silences-demo-claude.monster
│   │   │   │       └── silences-demo.monster
│   │   │   ├── support/
│   │   │   │   ├── login.rs
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
│   │   │   ├── plevir_fixture.rs
│   │   │   ├── print.rs
│   │   │   ├── record.rs
│   │   │   ├── record_cost.rs
│   │   │   ├── reduced_motion.rs
│   │   │   ├── scene_codec.rs
│   │   │   ├── scene_probe.rs
│   │   │   ├── sleep_wake.rs
│   │   │   └── view_source.rs
│   │   ├── web/
│   │   │   ├── index.html
│   │   │   └── Trunk.toml
│   │   └── Cargo.toml
│   └── urnadbapp/
│       ├── src/
│       │   ├── backend/
│       │   │   ├── sources/
│       │   │   │   ├── frames.rs
│       │   │   │   ├── mod.rs
│       │   │   │   ├── native.rs
│       │   │   │   └── reader.rs
│       │   │   ├── workers/
│       │   │   │   ├── embed.rs
│       │   │   │   ├── messages.rs
│       │   │   │   ├── mod.rs
│       │   │   │   ├── subprocess.rs
│       │   │   │   ├── web.rs
│       │   │   │   └── worker.rs
│       │   │   ├── benchmark.rs
│       │   │   └── mod.rs
│       │   ├── view/
│       │   │   ├── explorer/
│       │   │   │   ├── dialogs.rs
│       │   │   │   └── mod.rs
│       │   │   ├── chunks.rs
│       │   │   ├── field.rs
│       │   │   ├── graph.rs
│       │   │   ├── mod.rs
│       │   │   ├── open.rs
│       │   │   ├── overview.rs
│       │   │   ├── search.rs
│       │   │   └── stats.rs
│       │   ├── app.rs
│       │   ├── keyboard.rs
│       │   ├── lib.rs
│       │   ├── main.rs
│       │   ├── recents.rs
│       │   ├── renderer.rs
│       │   └── web_picker.rs
│       ├── tests/
│       │   ├── backend_integration.rs
│       │   ├── media_corpus.rs
│       │   └── web_reader_parity.rs
│       ├── web/
│       │   ├── index.html
│       │   └── Trunk.toml
│       ├── Cargo.lock
│       └── Cargo.toml
├── data/
│   └── media/
│       └── lottie/
│           ├── addbutton/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── animation.svg
│           ├── barmotion/
│           │   ├── animation.json
│           │   └── animation.lottie
│           ├── bottombar/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── animation.svg
│           ├── buttontap/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── animation.svg
│           ├── cardslist/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── animation.webm
│           ├── cardstack/
│           │   ├── animation.json
│           │   ├── animation.svg
│           │   └── animation.webm
│           ├── catsmiles/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── thumbnail.webp
│           ├── expandone/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── animation.svg
│           ├── expandtwo/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── animation.svg
│           ├── explosion/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   ├── animation.webm
│           │   ├── fireburst.lottie
│           │   ├── optimized.json
│           │   └── optimized.lottie
│           ├── girlscene/
│           │   ├── alternate.lottie
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   ├── animation.webm
│           │   ├── optimized.json
│           │   └── optimized.lottie
│           ├── moneyicon/
│           │   ├── animation.json
│           │   ├── animation.lottie
│           │   └── animation.svg
│           └── snakeloop/
│               ├── animation.gif
│               ├── animation.json
│               └── animation.lottie
├── demo/
│   ├── arcade/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── render.rs
│   │   │   ├── state.rs
│   │   │   └── ui.rs
│   │   └── Cargo.toml
│   ├── camera/
│   │   ├── src/
│   │   │   ├── components/
│   │   │   │   ├── chip.rs
│   │   │   │   ├── hud_panel.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── stat_card.rs
│   │   │   ├── render/
│   │   │   │   ├── camera.rs
│   │   │   │   ├── geometry.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── pipeline.rs
│   │   │   │   ├── scene.rs
│   │   │   │   └── shader.wgsl
│   │   │   ├── main.rs
│   │   │   └── theme.rs
│   │   └── Cargo.toml
│   ├── commit/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── inputs/
│   │   ├── src/
│   │   │   ├── gpu.rs
│   │   │   ├── main.rs
│   │   │   ├── palette.rs
│   │   │   ├── render.rs
│   │   │   └── state.rs
│   │   └── Cargo.toml
│   ├── intent/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── layers/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── palette.rs
│   │   │   ├── render.rs
│   │   │   └── ui.rs
│   │   └── Cargo.toml
│   ├── lottie/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── mobile/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── palette.rs
│   │   │   ├── render.rs
│   │   │   ├── state.rs
│   │   │   └── ui.rs
│   │   └── Cargo.toml
│   ├── mockup/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── player/
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── primes/
│   │   ├── src/
│   │   │   ├── simulation/
│   │   │   │   ├── coherence.rs
│   │   │   │   ├── grid.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── parameters.rs
│   │   │   │   ├── particle.rs
│   │   │   │   ├── physics.rs
│   │   │   │   ├── primes.rs
│   │   │   │   └── random.rs
│   │   │   ├── color.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   └── scene.rs
│   │   ├── Cargo.toml
│   │   ├── index.html
│   │   └── Trunk.toml
│   ├── shapes/
│   │   ├── src/
│   │   │   ├── cards_row1.rs
│   │   │   ├── cards_row2.rs
│   │   │   ├── main.rs
│   │   │   ├── render.rs
│   │   │   └── shapes.rs
│   │   └── Cargo.toml
│   └── ticker/
│       ├── src/
│       │   ├── lifecycle.rs
│       │   ├── main.rs
│       │   └── render.rs
│       └── Cargo.toml
├── docs/
│   ├── adr/
│   │   ├── generated/
│   │   │   ├── 0049-plev-ir-typed-and-verified.md
│   │   │   ├── 0051-materialized-surfaces-from-verified-ir.md
│   │   │   ├── 0052-proposers-propose-the-runtime-verifies.md
│   │   │   ├── 0067-adaptive-transforms-preserve-identity.md
│   │   │   └── 0070-generative-runtime-results.md
│   │   ├── importing/
│   │   │   ├── 0010-binary-animation-format-with-discovered-deltas.md
│   │   │   ├── 0011-import-foreign-formats-by-conversion-not-embedding.md
│   │   │   ├── 0012-transpiler-reports-every-unmapped-construct.md
│   │   │   ├── 0025-urnaui-media-frames-in-place-via-ffmpeg-subfile.md
│   │   │   ├── 0066-ui-asset-compiler.md
│   │   │   └── 0069-importers-target-plev-ir.md
│   │   ├── interface/
│   │   │   ├── 0009-touch-as-synthesized-pointer-events.md
│   │   │   ├── 0021-text-editing-core-follows-helix.md
│   │   │   ├── 0038-accessibility-projects-the-semantic-snapshot.md
│   │   │   ├── 0055-file-dialogs-and-drops-are-capabilities.md
│   │   │   ├── 0057-native-menus-are-another-view-of-capabilities.md
│   │   │   ├── 0058-native-dialogs-answer-the-same-contract.md
│   │   │   ├── 0059-tray-and-notifications-are-capabilities-out-of-the-window.md
│   │   │   ├── 0062-capture-is-a-job-the-person-turns-on.md
│   │   │   ├── 0063-a-link-is-not-a-person.md
│   │   │   ├── 0068-accessibility-is-production-ready-per-platform.md
│   │   │   ├── 0075-text-moves-by-screen-order-and-shapes-by-line.md
│   │   │   ├── 0076-the-keyboard-walks-the-projection.md
│   │   │   └── 0077-the-persons-text-and-clipboard-stay-the-persons.md
│   │   ├── recording/
│   │   │   ├── 0037-session-recording-replay-not-reexecution.md
│   │   │   ├── 0046-the-inspector-reads-a-session-never-runs-it.md
│   │   │   ├── 0050-sessions-become-regression-tests.md
│   │   │   └── 0065-scene-travels-as-data.md
│   │   ├── rendering/
│   │   │   ├── 0001-async-gpu-init-and-single-wasm-entry.md
│   │   │   ├── 0002-content-driven-layout-not-fixed-constants.md
│   │   │   ├── 0003-embed-every-font-weight-in-use.md
│   │   │   ├── 0004-linearize-colors-before-the-gpu.md
│   │   │   ├── 0005-measured-design-tokens-over-eyeballed-values.md
│   │   │   ├── 0006-one-text-style-for-measurement-and-drawing.md
│   │   │   ├── 0007-render-into-an-srgb-view-format.md
│   │   │   ├── 0008-render-on-demand-requires-explicit-invalidation.md
│   │   │   ├── 0014-motion-trails-by-position-history.md
│   │   │   ├── 0016-hidpi-projection.md
│   │   │   ├── 0024-glyph-raster-identity-and-atlas-isolation.md
│   │   │   ├── 0026-worker-results-wake-the-event-loop.md
│   │   │   ├── 0028-core-theme-tailwind-classes-as-recipes.md
│   │   │   ├── 0029-embed-an-app-view-by-frame-origin.md
│   │   │   ├── 0034-headless-render-shares-the-frame-path.md
│   │   │   ├── 0041-svg-export-reads-the-scene.md
│   │   │   ├── 0042-a-tick-asks-for-a-deadline.md
│   │   │   ├── 0054-gpu-device-loss-recovers-in-place.md
│   │   │   ├── 0060-a-covered-window-does-not-draw-and-wakes-new.md
│   │   │   ├── 0061-printing-is-the-frame-on-paper.md
│   │   │   ├── 0071-embedded-fonts-by-script.md
│   │   │   ├── 0073-design-for-ranges-not-screens-and-devices.md
│   │   │   └── 0074-a-renderer-upgrade-is-proven-by-a-full-capture.md
│   │   ├── semantics/
│   │   │   ├── 0032-semantic-contract-as-pure-projection.md
│   │   │   ├── 0033-actions-carry-observed-state.md
│   │   │   ├── 0035-semantic-binding-beside-the-scene.md
│   │   │   ├── 0036-commit-checks-the-index-it-observed.md
│   │   │   ├── 0039-agents-act-through-the-semantic-contract.md
│   │   │   ├── 0040-lean-semantic-form-and-one-redaction.md
│   │   │   ├── 0043-a-delta-carries-the-node-a-refusal-names-who-acted.md
│   │   │   ├── 0044-a-terminal-stage-carries-the-revision.md
│   │   │   └── 0047-an-act-rests-on-what-its-actor-was-shown.md
│   │   ├── structure/
│   │   │   ├── 0013-workspace-engine-at-root-libs-in-crates-demos-in-examples.md
│   │   │   ├── 0015-srp-modularization.md
│   │   │   ├── 0020-git-backend-gix-reads-cli-mutations.md
│   │   │   ├── 0022-one-knowledge-tree-and-a-minimal-root.md
│   │   │   ├── 0023-official-app-pattern.md
│   │   │   ├── 0027-comps-one-design-system-crate.md
│   │   │   ├── 0030-showcase-one-page-per-component.md
│   │   │   ├── 0053-the-silences-app-is-the-fixed-screen.md
│   │   │   ├── 0056-windows-are-hosts-the-app-owns-the-state.md
│   │   │   └── 0072-ai-native-single-process.md
│   │   ├── workflows/
│   │   │   ├── 0017-error-handling-lint-cleanup.md
│   │   │   ├── 0018-narrate-hot-reload.md
│   │   │   ├── 0019-shader-hot-reload.md
│   │   │   ├── 0031-hot-reload-on-by-default-in-gpu-context.md
│   │   │   ├── 0045-regression-by-meaning-before-pixels.md
│   │   │   ├── 0048-view-source-for-pixels.md
│   │   │   └── 0064-what-identical-means.md
│   │   ├── README.md
│   │   └── TEMPLATE.md
│   ├── img/
│   │   └── task-69-checkpoint-b/
│   │       ├── 02-fixed-detected.png
│   │       ├── 03-generated-rules.png
│   │       ├── 04-generated-claude.png
│   │       ├── 05-after-task.png
│   │       ├── r2-02-fixed-detected.png
│   │       ├── r2-03-generated-rules.png
│   │       ├── r2-04-generated-claude.png
│   │       └── r2-05-after-task.png
│   ├── log/
│   │   ├── changes/
│   │   │   ├── gap-1.md
│   │   │   ├── task-01.md
│   │   │   ├── task-02.md
│   │   │   ├── task-03.md
│   │   │   ├── task-04.md
│   │   │   ├── task-05.md
│   │   │   ├── task-06.md
│   │   │   ├── task-07.md
│   │   │   ├── task-08.md
│   │   │   ├── task-09.md
│   │   │   ├── task-10.md
│   │   │   ├── task-11.md
│   │   │   ├── task-12.md
│   │   │   ├── task-13.md
│   │   │   ├── task-14.md
│   │   │   ├── task-15.md
│   │   │   ├── task-16.md
│   │   │   ├── task-17.md
│   │   │   ├── task-18.md
│   │   │   ├── task-19.md
│   │   │   ├── task-20.md
│   │   │   ├── task-21.md
│   │   │   ├── task-22.md
│   │   │   ├── task-23.md
│   │   │   ├── task-24.md
│   │   │   ├── task-25.md
│   │   │   ├── task-26.md
│   │   │   ├── task-27.md
│   │   │   ├── task-28.md
│   │   │   ├── task-29.md
│   │   │   ├── task-30.md
│   │   │   ├── task-31.md
│   │   │   ├── task-32.md
│   │   │   ├── task-33.md
│   │   │   ├── task-34.md
│   │   │   ├── task-35.md
│   │   │   ├── task-36.md
│   │   │   ├── task-37.md
│   │   │   ├── task-38.md
│   │   │   ├── task-42.md
│   │   │   ├── task-43.md
│   │   │   ├── task-44.md
│   │   │   ├── task-45.md
│   │   │   ├── task-46.md
│   │   │   ├── task-47.md
│   │   │   ├── task-48.md
│   │   │   ├── task-49.md
│   │   │   ├── task-50.md
│   │   │   ├── task-51.md
│   │   │   ├── task-52.md
│   │   │   └── task-ref.md
│   │   ├── learnings/
│   │   │   ├── comps.md
│   │   │   ├── core-buttons.md
│   │   │   ├── general.md
│   │   │   ├── generative-runtime.md
│   │   │   ├── hot-reload.md
│   │   │   └── showcase-pages.md
│   │   ├── steps/
│   │   │   ├── task-01.md
│   │   │   ├── task-02.md
│   │   │   ├── task-03.md
│   │   │   ├── task-04.md
│   │   │   ├── task-05.md
│   │   │   ├── task-06.md
│   │   │   ├── task-07.md
│   │   │   ├── task-08.md
│   │   │   ├── task-09.md
│   │   │   ├── task-10.md
│   │   │   ├── task-11.md
│   │   │   ├── task-12.md
│   │   │   ├── task-13.md
│   │   │   ├── task-14.md
│   │   │   ├── task-15.md
│   │   │   ├── task-16.md
│   │   │   ├── task-17.md
│   │   │   ├── task-18-android-device-test.md
│   │   │   ├── task-19-ios-simulator-test.md
│   │   │   ├── task-20-wasm-visual-validation.md
│   │   │   ├── task-21.md
│   │   │   ├── task-22.md
│   │   │   ├── task-23.md
│   │   │   ├── task-24.md
│   │   │   ├── task-25-benchmarks.md
│   │   │   ├── task-26-arxiv-paper.md
│   │   │   ├── task-27.md
│   │   │   ├── task-28.md
│   │   │   ├── task-29.md
│   │   │   ├── task-30-accessibility.md
│   │   │   ├── task-31-vector-paths.md
│   │   │   ├── task-32-text-upgrade-assessment.md
│   │   │   ├── task-33-wasm-plugins.md
│   │   │   ├── task-34-exploration-extraction.md
│   │   │   ├── task-35-spring-analytical-solver.md
│   │   │   ├── task-36-signal-hardening.md
│   │   │   ├── task-37-animation-enhancements.md
│   │   │   ├── task-38-event-batching.md
│   │   │   ├── task-39-scene-cache-memoization.md
│   │   │   ├── task-41-license-and-public-release.md
│   │   │   ├── task-42-plev-event-dispatch.md
│   │   │   ├── task-43-monster-codec.md
│   │   │   ├── task-44-lot-lottie-importer.md
│   │   │   ├── task-45-parser-transpiler.md
│   │   │   ├── task-46-rope-editing-core.md
│   │   │   ├── task-47-git-backend.md
│   │   │   ├── task-48-ide-git-client.md
│   │   │   ├── task-49-prime-creatures.md
│   │   │   ├── task-50-showcase-multiplatform.md
│   │   │   ├── task-51-workspace-restructure.md
│   │   │   ├── task-52-srp-modularization.md
│   │   │   ├── task-53-editor-and-git-in-the-showcase.md
│   │   │   ├── task-54-core-theme-buttons-poc.md
│   │   │   ├── task-55-showcase-one-page-per-component.md
│   │   │   ├── task-56-hot-reload.md
│   │   │   ├── task-57-semantic-contract.md
│   │   │   ├── task-57b-lean-wire-form-and-one-redaction.md
│   │   │   ├── task-58-actions-preconditions-receipts.md
│   │   │   ├── task-58b-decline-a-confirmation.md
│   │   │   ├── task-59-semantic-binding-and-one-path.md
│   │   │   ├── task-60-accessibility-projection.md
│   │   │   ├── task-61-agent-surface.md
│   │   │   ├── task-62-ide-human-agent-proof.md
│   │   │   ├── task-63-flight-recorder.md
│   │   │   ├── task-64-session-inspector-and-tests.md
│   │   │   ├── task-65-pixel-to-source.md
│   │   │   ├── task-66-headless-first-class.md
│   │   │   ├── task-66b-svg-export.md
│   │   │   ├── task-67-structural-regression.md
│   │   │   ├── task-68-plev-ir-and-verifier.md
│   │   │   ├── task-69-ephemeral-generative-surfaces.md
│   │   │   ├── task-70-adaptive-transformations.md
│   │   │   ├── task-71-compiler-target.md
│   │   │   ├── task-72-ui-asset-compiler.md
│   │   │   ├── task-73-incremental-dependency-system.md
│   │   │   ├── task-73-measurements.md
│   │   │   ├── task-74-text-and-editing.md
│   │   │   ├── task-75-platform-accessibility.md
│   │   │   ├── task-76-platform-integration.md
│   │   │   ├── task-77-visual-determinism.md
│   │   │   ├── task-78-ai-native-apps.md
│   │   │   ├── task-79-scene-over-socket.md
│   │   │   ├── task-80-proof-qualitatively-different-software.md
│   │   │   ├── task-81-select-scrolling-panel.md
│   │   │   ├── task-82-caret-blink-render-on-demand.md
│   │   │   ├── task-83-delta-carries-node-refusal-names-actor.md
│   │   │   ├── task-84-ide-refresh-never-goes-back.md
│   │   │   ├── task-85-the-commit-a-receipt-announces-is-projected.md
│   │   │   ├── task-86-the-ide-opens-the-agent-port.md
│   │   │   ├── task-87-a-terminal-stage-carries-the-revision.md
│   │   │   ├── task-88-the-lab-measures-what-the-agent-rested-on.md
│   │   │   ├── task-89-an-act-rests-only-on-what-was-shown.md
│   │   │   ├── task-90-a-refusal-names-who-acted-since-the-look.md
│   │   │   ├── task-91-the-dispatcher-knows-when-the-agent-looked.md
│   │   │   ├── task-92-the-agent-asks-the-person.md
│   │   │   ├── task-93-the-lab-person-answers.md
│   │   │   ├── task-94-the-refusal-points-at-the-question.md
│   │   │   ├── task-95-the-todo-projects-its-items-as-a-list.md
│   │   │   ├── task-96-ide-generated-surface.md
│   │   │   ├── task-97-code-and-diff-in-the-ir.md
│   │   │   └── task-ref-research-briefing.md
│   │   ├── changelog.md
│   │   ├── generative-runtime-program.md
│   │   ├── journal.md
│   │   ├── README.md
│   │   ├── responsiveness-multiplatform-and-fidelity.md
│   │   └── truth.md
│   ├── pub/
│   │   ├── arxiv-paper-draft.md
│   │   ├── arxiv-paper-outline.md
│   │   ├── benchmark-results.md
│   │   ├── brief-strengths.md
│   │   └── generative-runtime-thesis.md
│   ├── ref/
│   │   ├── accessibility.md
│   │   ├── android-emulator-deploy.md
│   │   ├── animation-motion.md
│   │   ├── animation-pattern-lerp.md
│   │   ├── autonomy.md
│   │   ├── build-and-serve-the-web-target.md
│   │   ├── charts-visualization.md
│   │   ├── clippy-zero-warnings.md
│   │   ├── code-against-the-plev-engine.md
│   │   ├── competitive-positioning.md
│   │   ├── competitors-by-tier.md
│   │   ├── competitors.md
│   │   ├── component-catalog.md
│   │   ├── component-design.md
│   │   ├── dsl-narrate-design.md
│   │   ├── effects-architecture.md
│   │   ├── emulators-wasm-runtimes.md
│   │   ├── extracted-patterns.md
│   │   ├── extraction-fase-ef.md
│   │   ├── generative-runtime-guidelines.md
│   │   ├── hot-reload-design.md
│   │   ├── input-system-design.md
│   │   ├── integration-candidates.md
│   │   ├── ios-build.md
│   │   ├── layer-system.md
│   │   ├── layout-engine.md
│   │   ├── linebender-ecosystem.md
│   │   ├── makepad-gap-analysis.md
│   │   ├── math-physics-geometry.md
│   │   ├── mobile-specifics.md
│   │   ├── mon-experiment-notes.md
│   │   ├── monster-format-v0.md
│   │   ├── orchestrate-coding-agents-on-this-repo.md
│   │   ├── parley-vs-cosmic-text.md
│   │   ├── pattern-extraction-parley-lyon-glam.md
│   │   ├── pattern-extraction-tui-apps.md
│   │   ├── plev-vs-makepad-report.md
│   │   ├── signal-system-design.md
│   │   ├── srp-modularization.md
│   │   ├── technology-radar.md
│   │   ├── touch-gesture-design.md
│   │   ├── tui-patterns.md
│   │   ├── validate-visuals-by-pixel.md
│   │   ├── view-trait-design.md
│   │   ├── wasm-plugin-architecture.md
│   │   ├── wasm-tooling.md
│   │   └── wasm-webgpu-validation.md
│   ├── ATLAS.toml
│   ├── README.md
│   └── TERMS.toml
├── fuzz/
│   └── .gitkeep
├── pkgs/
│   └── .gitkeep
├── rust/
│   ├── accord/
│   │   ├── src/
│   │   │   ├── assembly.rs
│   │   │   ├── controls.rs
│   │   │   ├── decision.rs
│   │   │   ├── dispatch.rs
│   │   │   ├── encoding.rs
│   │   │   ├── failures.rs
│   │   │   ├── identity.rs
│   │   │   ├── lib.rs
│   │   │   ├── manifest.rs
│   │   │   ├── mutation.rs
│   │   │   ├── notation.rs
│   │   │   ├── property.rs
│   │   │   ├── receipts.rs
│   │   │   ├── redactor.rs
│   │   │   ├── requests.rs
│   │   │   ├── revision.rs
│   │   │   ├── snapshot.rs
│   │   │   ├── taxonomy.rs
│   │   │   └── validate.rs
│   │   ├── tests/
│   │   │   ├── assembly.rs
│   │   │   ├── decision.rs
│   │   │   ├── dispatch.rs
│   │   │   ├── encoding.rs
│   │   │   ├── identity.rs
│   │   │   ├── mutation.rs
│   │   │   ├── property.rs
│   │   │   ├── receipts.rs
│   │   │   ├── requests.rs
│   │   │   └── snapshot.rs
│   │   └── Cargo.toml
│   ├── claude/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── reply.rs
│   │   │   └── run.rs
│   │   ├── tests/
│   │   │   ├── fakes/
│   │   │   │   └── claude
│   │   │   └── cli.rs
│   │   └── Cargo.toml
│   ├── editor/
│   │   ├── benches/
│   │   │   └── edit.rs
│   │   ├── src/
│   │   │   ├── document/
│   │   │   │   ├── history.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── movement.rs
│   │   │   │   ├── selection.rs
│   │   │   │   └── transaction.rs
│   │   │   ├── tests/
│   │   │   │   ├── bidi.rs
│   │   │   │   └── mod.rs
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   └── properties.rs
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
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   └── text_probe.rs
│   │   │   ├── compose/
│   │   │   │   ├── animation/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── clock.rs
│   │   │   │   │   │   ├── easing.rs
│   │   │   │   │   │   ├── keyframe.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── reduced_motion.rs
│   │   │   │   │   │   ├── spring.rs
│   │   │   │   │   │   └── tween.rs
│   │   │   │   │   ├── easing.rs
│   │   │   │   │   ├── keyframe.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── spring.rs
│   │   │   │   │   └── tween.rs
│   │   │   │   ├── builder/
│   │   │   │   │   ├── element/
│   │   │   │   │   │   ├── builders.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── emit.rs
│   │   │   │   │   │   ├── intent.rs
│   │   │   │   │   │   ├── layout.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── styles.rs
│   │   │   │   │   ├── conversions.rs
│   │   │   │   │   ├── emit.rs
│   │   │   │   │   ├── layout_pipeline.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── output.rs
│   │   │   │   │   ├── render.rs
│   │   │   │   │   └── style.rs
│   │   │   │   ├── layout/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── flex.rs
│   │   │   │   │   │   ├── measure.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── percent.rs
│   │   │   │   │   │   └── perf.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── solver.rs
│   │   │   │   │   └── style.rs
│   │   │   │   ├── markup_runtime/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── extraction.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── extraction.rs
│   │   │   │   │   ├── keywords.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── modifiers.rs
│   │   │   │   │   ├── parser.rs
│   │   │   │   │   └── tokenizer.rs
│   │   │   │   ├── view/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── lifecycle.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── context.rs
│   │   │   │   │   ├── lifecycle.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── views.rs
│   │   │   │   └── mod.rs
│   │   │   ├── control/
│   │   │   │   ├── accessibility/
│   │   │   │   │   ├── action/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── present/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── tree/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── bridge.rs
│   │   │   │   │   ├── focus.rs
│   │   │   │   │   ├── ids.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── reduced_motion.rs
│   │   │   │   │   └── role.rs
│   │   │   │   ├── input/
│   │   │   │   │   ├── gesture/
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── basic.rs
│   │   │   │   │   │   │   ├── complex.rs
│   │   │   │   │   │   │   └── mod.rs
│   │   │   │   │   │   ├── handlers.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── recognizer.rs
│   │   │   │   │   │   └── state.rs
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── hit.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── touch/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── synthesis.rs
│   │   │   │   │   ├── dispatch.rs
│   │   │   │   │   ├── drop.rs
│   │   │   │   │   ├── events.rs
│   │   │   │   │   ├── handlers.rs
│   │   │   │   │   ├── hit_test.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── scroll.rs
│   │   │   │   ├── keymap/
│   │   │   │   │   ├── bindings.rs
│   │   │   │   │   ├── context.rs
│   │   │   │   │   ├── keystroke.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── shortcuts.rs
│   │   │   │   │   └── winit.rs
│   │   │   │   └── mod.rs
│   │   │   ├── drawing/
│   │   │   │   ├── compositor/
│   │   │   │   │   ├── layer/
│   │   │   │   │   │   ├── geometry.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── texture.rs
│   │   │   │   │   ├── binding.rs
│   │   │   │   │   ├── clip.rs
│   │   │   │   │   ├── drawing.rs
│   │   │   │   │   ├── layer_operations.rs
│   │   │   │   │   ├── memory.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── scene.rs
│   │   │   │   │   ├── sequence.rs
│   │   │   │   │   ├── stats.rs
│   │   │   │   │   ├── tests.rs
│   │   │   │   │   └── vertex.rs
│   │   │   │   ├── effects/
│   │   │   │   │   ├── apply.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── parameters.rs
│   │   │   │   │   ├── pipelines.rs
│   │   │   │   │   ├── sequencer.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── gpu/
│   │   │   │   │   ├── shaders/
│   │   │   │   │   │   ├── backdrop.wgsl
│   │   │   │   │   │   ├── blur.wgsl
│   │   │   │   │   │   ├── composite.wgsl
│   │   │   │   │   │   ├── image.wgsl
│   │   │   │   │   │   ├── quad.wgsl
│   │   │   │   │   │   ├── rect_sdf.wgsl
│   │   │   │   │   │   ├── shadow.wgsl
│   │   │   │   │   │   ├── shadow_analytic.wgsl
│   │   │   │   │   │   └── text.wgsl
│   │   │   │   │   ├── config.rs
│   │   │   │   │   ├── context.rs
│   │   │   │   │   ├── image.rs
│   │   │   │   │   ├── loss.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── pipelines.rs
│   │   │   │   │   ├── presets.rs
│   │   │   │   │   ├── recover.rs
│   │   │   │   │   ├── surface.rs
│   │   │   │   │   ├── tests.rs
│   │   │   │   │   ├── texture_pool.rs
│   │   │   │   │   └── vec.rs
│   │   │   │   ├── headless/
│   │   │   │   │   ├── svg/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── shape.rs
│   │   │   │   │   │   ├── tests.rs
│   │   │   │   │   │   ├── text.rs
│   │   │   │   │   │   └── write.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── readback.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── path/
│   │   │   │   │   ├── builder.rs
│   │   │   │   │   ├── mesh.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── tessellation.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── text/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── measure.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── raster.rs
│   │   │   │   │   ├── atlas.rs
│   │   │   │   │   ├── backend.rs
│   │   │   │   │   ├── cache.rs
│   │   │   │   │   ├── cluster_cache.rs
│   │   │   │   │   ├── fonts.rs
│   │   │   │   │   ├── measure.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── probe.rs
│   │   │   │   │   ├── system.rs
│   │   │   │   │   └── vertex.rs
│   │   │   │   └── mod.rs
│   │   │   ├── runtime/
│   │   │   │   ├── hot_reload/
│   │   │   │   │   ├── literals/
│   │   │   │   │   │   ├── write/
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   └── tests.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── markup.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── shader.rs
│   │   │   │   │   ├── source.rs
│   │   │   │   │   ├── tests.rs
│   │   │   │   │   └── view_source.rs
│   │   │   │   ├── perf/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── stages.rs
│   │   │   │   │   ├── gpu_time.rs
│   │   │   │   │   ├── hud.rs
│   │   │   │   │   ├── memory.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── samples.rs
│   │   │   │   │   └── stages.rs
│   │   │   │   ├── platform/
│   │   │   │   │   ├── capture/
│   │   │   │   │   │   ├── ffmpeg/
│   │   │   │   │   │   │   ├── args.rs
│   │   │   │   │   │   │   ├── child.rs
│   │   │   │   │   │   │   └── mod.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── links/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── source.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── menu/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── native.rs
│   │   │   │   │   ├── print/
│   │   │   │   │   │   ├── cups.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── tray/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── native.rs
│   │   │   │   │   ├── alerts.rs
│   │   │   │   │   ├── clipboard.rs
│   │   │   │   │   ├── files.rs
│   │   │   │   │   ├── ime.rs
│   │   │   │   │   ├── lifecycle.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── notify.rs
│   │   │   │   ├── signal/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── advanced.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── api.rs
│   │   │   │   │   ├── execution.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── runtime.rs
│   │   │   │   ├── window/
│   │   │   │   │   ├── events.rs
│   │   │   │   │   ├── frame.rs
│   │   │   │   │   ├── host.rs
│   │   │   │   │   ├── hot_reload.rs
│   │   │   │   │   ├── launch.rs
│   │   │   │   │   ├── lifecycle.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── render.rs
│   │   │   │   │   ├── render_passes.rs
│   │   │   │   │   ├── sleep.rs
│   │   │   │   │   └── windows.rs
│   │   │   │   └── mod.rs
│   │   │   ├── styling/
│   │   │   │   ├── theme/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── contrast.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── oklch.rs
│   │   │   │   │   │   └── scales.rs
│   │   │   │   │   ├── color_space.rs
│   │   │   │   │   ├── hoff.rs
│   │   │   │   │   ├── hoff_core.rs
│   │   │   │   │   ├── intent.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── scales.rs
│   │   │   │   │   └── tokens.rs
│   │   │   │   ├── color.rs
│   │   │   │   └── mod.rs
│   │   │   ├── widgets/
│   │   │   │   ├── buttons/
│   │   │   │   │   ├── button.rs
│   │   │   │   │   ├── icon_button.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── press.rs
│   │   │   │   │   ├── submit_button.rs
│   │   │   │   │   └── tokens.rs
│   │   │   │   ├── charts/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── donut.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── area.rs
│   │   │   │   │   ├── axis.rs
│   │   │   │   │   ├── bars.rs
│   │   │   │   │   ├── donut.rs
│   │   │   │   │   ├── draw.rs
│   │   │   │   │   ├── line.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── content/
│   │   │   │   │   ├── avatar.rs
│   │   │   │   │   ├── badge.rs
│   │   │   │   │   ├── card.rs
│   │   │   │   │   ├── chip.rs
│   │   │   │   │   ├── code_block.rs
│   │   │   │   │   ├── empty_state.rs
│   │   │   │   │   ├── list.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── panel.rs
│   │   │   │   │   ├── separator.rs
│   │   │   │   │   ├── skeleton.rs
│   │   │   │   │   ├── stat.rs
│   │   │   │   │   ├── table.rs
│   │   │   │   │   └── tree.rs
│   │   │   │   ├── editor/
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── bidi.rs
│   │   │   │   │   │   ├── large.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── config.rs
│   │   │   │   │   ├── input.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── view.rs
│   │   │   │   ├── feedback/
│   │   │   │   │   ├── context_menu.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── modal.rs
│   │   │   │   │   ├── progress.rs
│   │   │   │   │   ├── scrollbar.rs
│   │   │   │   │   ├── spinner.rs
│   │   │   │   │   ├── toast.rs
│   │   │   │   │   └── tooltip.rs
│   │   │   │   ├── form/
│   │   │   │   │   ├── select/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── panel.rs
│   │   │   │   │   ├── text_input/
│   │   │   │   │   │   ├── tests/
│   │   │   │   │   │   │   ├── bidi.rs
│   │   │   │   │   │   │   ├── buffer.rs
│   │   │   │   │   │   │   ├── clipboard.rs
│   │   │   │   │   │   │   ├── component.rs
│   │   │   │   │   │   │   ├── ime.rs
│   │   │   │   │   │   │   └── mod.rs
│   │   │   │   │   │   ├── buffer.rs
│   │   │   │   │   │   ├── component.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── visual.rs
│   │   │   │   │   ├── checkbox.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── slider.rs
│   │   │   │   │   ├── switch.rs
│   │   │   │   │   └── text_field.rs
│   │   │   │   ├── graph/
│   │   │   │   │   ├── geometry.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── icons/
│   │   │   │   │   ├── data.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── svg.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── nav/
│   │   │   │   │   ├── sidebar/
│   │   │   │   │   │   ├── input.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── render.rs
│   │   │   │   │   ├── breadcrumb.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── nav_link.rs
│   │   │   │   │   ├── panel_header.rs
│   │   │   │   │   ├── split_pane.rs
│   │   │   │   │   └── tabs.rs
│   │   │   │   ├── overlay/
│   │   │   │   │   ├── item.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── tests/
│   │   │   │   │   ├── widgets/
│   │   │   │   │   │   ├── semantics/
│   │   │   │   │   │   │   ├── buttons.rs
│   │   │   │   │   │   │   ├── content.rs
│   │   │   │   │   │   │   ├── feedback.rs
│   │   │   │   │   │   │   ├── form.rs
│   │   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   │   ├── nav.rs
│   │   │   │   │   │   │   └── roving.rs
│   │   │   │   │   │   ├── buttons.rs
│   │   │   │   │   │   ├── content.rs
│   │   │   │   │   │   ├── feedback.rs
│   │   │   │   │   │   ├── fixtures.rs
│   │   │   │   │   │   ├── form.rs
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   └── nav.rs
│   │   │   │   │   ├── components.rs
│   │   │   │   │   ├── focus.rs
│   │   │   │   │   ├── matrix.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── wake.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── recipe.rs
│   │   │   │   ├── shell.rs
│   │   │   │   ├── wake.rs
│   │   │   │   └── widget.rs
│   │   │   ├── error.rs
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   ├── fakes/
│   │   │   │   ├── ffmpeg
│   │   │   │   ├── ffmpeg-mute
│   │   │   │   ├── ffmpeg-refuses
│   │   │   │   ├── lp
│   │   │   │   ├── lpstat
│   │   │   │   └── lpstat-none
│   │   │   ├── fixtures/
│   │   │   │   ├── text_corpus/
│   │   │   │   │   ├── 01-latin-diacritics.txt
│   │   │   │   │   ├── 02-latin-ligatures-kerning.txt
│   │   │   │   │   ├── 03-combining-marks.txt
│   │   │   │   │   ├── 04-arabic.txt
│   │   │   │   │   ├── 05-hebrew.txt
│   │   │   │   │   ├── 06-devanagari.txt
│   │   │   │   │   ├── 07-chinese.txt
│   │   │   │   │   ├── 08-japanese.txt
│   │   │   │   │   ├── 09-korean.txt
│   │   │   │   │   ├── 10-emoji.txt
│   │   │   │   │   ├── 11-bidi-mixed.txt
│   │   │   │   │   └── README.txt
│   │   │   │   ├── blur_at_scale_1.png
│   │   │   │   └── text_audit.txt
│   │   │   ├── alerts.rs
│   │   │   ├── asset_cost.rs
│   │   │   ├── blur_scale.rs
│   │   │   ├── capture.rs
│   │   │   ├── device_loss.rs
│   │   │   ├── editor.rs
│   │   │   ├── headless_determinism.rs
│   │   │   ├── links.rs
│   │   │   ├── menu.rs
│   │   │   ├── notify.rs
│   │   │   ├── print.rs
│   │   │   ├── sleep_wake.rs
│   │   │   ├── text_audit.rs
│   │   │   ├── text_drawn_extent.rs
│   │   │   ├── text_raster_pixels.rs
│   │   │   ├── tray.rs
│   │   │   ├── wgsl.rs
│   │   │   └── windows.rs
│   │   ├── build.rs
│   │   └── Cargo.toml
│   ├── gitdrv/
│   │   ├── src/
│   │   │   ├── client.rs
│   │   │   ├── diff.rs
│   │   │   ├── domain.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── repo.rs
│   │   │   └── status.rs
│   │   ├── tests/
│   │   │   ├── log_cost.rs
│   │   │   └── real_repo.rs
│   │   └── Cargo.toml
│   ├── import/
│   │   ├── benches/
│   │   │   ├── gpui.rs
│   │   │   └── lottie.rs
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   ├── convert.rs
│   │   │   │   └── transpile.rs
│   │   │   ├── emit/
│   │   │   │   ├── plevir/
│   │   │   │   │   ├── control.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── props.rs
│   │   │   │   ├── builder.rs
│   │   │   │   └── mod.rs
│   │   │   ├── formats/
│   │   │   │   ├── figma/
│   │   │   │   │   ├── look.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── parse.rs
│   │   │   │   │   └── resolve.rs
│   │   │   │   ├── gpui/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── node.rs
│   │   │   │   │   ├── parse.rs
│   │   │   │   │   └── resolve.rs
│   │   │   │   ├── html/
│   │   │   │   │   ├── css.rs
│   │   │   │   │   ├── frame.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── parse.rs
│   │   │   │   │   └── resolve.rs
│   │   │   │   ├── lottie/
│   │   │   │   │   ├── convert.rs
│   │   │   │   │   ├── geometry.rs
│   │   │   │   │   ├── keyframe.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── render.rs
│   │   │   │   │   └── schema.rs
│   │   │   │   ├── react/
│   │   │   │   │   ├── css.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── parse.rs
│   │   │   │   │   ├── resolve.rs
│   │   │   │   │   └── sass.rs
│   │   │   │   ├── svg/
│   │   │   │   │   ├── convert.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── tessellate.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   └── mod.rs
│   │   │   ├── lib.rs
│   │   │   ├── report.rs
│   │   │   ├── tokens.rs
│   │   │   └── tree.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   ├── figma/
│   │   │   │   │   ├── expected.ir.json
│   │   │   │   │   ├── expected.report.json
│   │   │   │   │   ├── expected.rs
│   │   │   │   │   └── settings.json
│   │   │   │   ├── gpui/
│   │   │   │   │   ├── expected.ir.json
│   │   │   │   │   ├── expected.report.json
│   │   │   │   │   ├── expected.rs
│   │   │   │   │   └── separator.rs
│   │   │   │   ├── html/
│   │   │   │   │   ├── expected.ir.json
│   │   │   │   │   ├── expected.report.json
│   │   │   │   │   ├── expected.rs
│   │   │   │   │   └── note-card.html
│   │   │   │   └── react/
│   │   │   │       ├── expected.ir.json
│   │   │   │       ├── expected.report.json
│   │   │   │       ├── expected.rs
│   │   │   │       ├── index.tsx
│   │   │   │       ├── note-card-variables.sass
│   │   │   │       └── NoteCard.module.sass
│   │   │   ├── support/
│   │   │   │   ├── checks.rs
│   │   │   │   └── mod.rs
│   │   │   ├── emitted_compiles.rs
│   │   │   ├── figma.rs
│   │   │   ├── golden.rs
│   │   │   ├── html.rs
│   │   │   ├── pixels.rs
│   │   │   └── plevir.rs
│   │   └── Cargo.toml
│   ├── llmrun/
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   └── llm_bench.rs
│   │   │   ├── models/
│   │   │   │   ├── burn/
│   │   │   │   │   ├── cache.rs
│   │   │   │   │   ├── load.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── model.rs
│   │   │   │   │   └── rotary.rs
│   │   │   │   ├── candle.rs
│   │   │   │   └── mod.rs
│   │   │   ├── prompt/
│   │   │   │   ├── chat.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── tokenizer.rs
│   │   │   ├── engine.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   └── stream.rs
│   │   ├── tests/
│   │   │   ├── engine.rs
│   │   │   └── real.rs
│   │   ├── Cargo.lock
│   │   ├── Cargo.toml
│   │   └── NOTICE
│   ├── macros/
│   │   ├── src/
│   │   │   ├── markup/
│   │   │   │   ├── codegen/
│   │   │   │   │   ├── generators.rs
│   │   │   │   │   ├── interpolation.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── tests.rs
│   │   │   │   ├── parse/
│   │   │   │   │   ├── keywords/
│   │   │   │   │   │   ├── mod.rs
│   │   │   │   │   │   ├── suggest.rs
│   │   │   │   │   │   └── tests.rs
│   │   │   │   │   ├── tests/
│   │   │   │   │   │   ├── errors.rs
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── block_item.rs
│   │   │   │   │   ├── element.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── modifier.rs
│   │   │   │   │   └── value.rs
│   │   │   │   └── mod.rs
│   │   │   ├── lib.rs
│   │   │   └── view.rs
│   │   └── Cargo.toml
│   ├── markup/
│   │   ├── src/
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   └── integration.rs
│   │   └── Cargo.toml
│   ├── motion/
│   │   ├── benches/
│   │   │   └── codec.rs
│   │   ├── src/
│   │   │   ├── discover/
│   │   │   │   ├── tests/
│   │   │   │   │   ├── discover.rs
│   │   │   │   │   ├── discover_operations.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── fit.rs
│   │   │   │   └── mod.rs
│   │   │   ├── encoding/
│   │   │   │   ├── read/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── section.rs
│   │   │   │   ├── tests/
│   │   │   │   │   ├── asset_path.rs
│   │   │   │   │   ├── golden.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── operations.rs
│   │   │   │   │   ├── properties.rs
│   │   │   │   │   ├── quantize.rs
│   │   │   │   │   ├── read.rs
│   │   │   │   │   ├── read_malformed.rs
│   │   │   │   │   └── write.rs
│   │   │   │   ├── asset_path.rs
│   │   │   │   ├── container.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── quantize.rs
│   │   │   │   ├── validate.rs
│   │   │   │   └── write.rs
│   │   │   ├── mappings/
│   │   │   │   ├── lower.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── raise.rs
│   │   │   │   └── tests.rs
│   │   │   ├── optimize/
│   │   │   │   ├── tests/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── optimize.rs
│   │   │   │   │   └── optimize_pipeline.rs
│   │   │   │   └── mod.rs
│   │   │   ├── play/
│   │   │   │   ├── tests/
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── play.rs
│   │   │   │   │   └── play_operations.rs
│   │   │   │   ├── evaluation.rs
│   │   │   │   └── mod.rs
│   │   │   ├── timeline/
│   │   │   │   ├── tests/
│   │   │   │   │   ├── easing.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   └── timeline.rs
│   │   │   │   ├── easing.rs
│   │   │   │   └── mod.rs
│   │   │   └── lib.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   ├── golden_v0_minimal.monster
│   │   │   │   └── golden_v0_ops.monster
│   │   │   ├── gate_bytes/
│   │   │   │   ├── dense.rs
│   │   │   │   ├── girl.rs
│   │   │   │   ├── keyframes.rs
│   │   │   │   ├── lottie.rs
│   │   │   │   └── main.rs
│   │   │   ├── gate_discover.rs
│   │   │   └── raise.rs
│   │   └── Cargo.toml
│   ├── offers/
│   │   ├── src/
│   │   │   ├── compose/
│   │   │   │   ├── chain.rs
│   │   │   │   ├── composite.rs
│   │   │   │   ├── detect.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── offer.rs
│   │   │   ├── review/
│   │   │   │   ├── mod.rs
│   │   │   │   └── tests.rs
│   │   │   ├── adapt.rs
│   │   │   ├── change.rs
│   │   │   ├── error.rs
│   │   │   ├── ids.rs
│   │   │   ├── lib.rs
│   │   │   ├── list.rs
│   │   │   ├── llm.rs
│   │   │   ├── operations.rs
│   │   │   ├── parts.rs
│   │   │   ├── prompt.rs
│   │   │   ├── reshape.rs
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
│   ├── plevir/
│   │   ├── assets/
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
│   │   │   ├── capabilities.rs
│   │   │   ├── decode.rs
│   │   │   ├── error.rs
│   │   │   ├── ir.rs
│   │   │   ├── item.rs
│   │   │   ├── lib.rs
│   │   │   ├── policy.rs
│   │   │   ├── schema.rs
│   │   │   ├── token.rs
│   │   │   └── vocabulary.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   ├── idestudio-commit-form.ir.json
│   │   │   │   ├── idestudio-review-commit.ir.json
│   │   │   │   ├── idestudio-workbench.snapshot.json
│   │   │   │   ├── silences-review-claude.ir.json
│   │   │   │   ├── uigallery-files.snapshot.json
│   │   │   │   ├── uigallery-silences.snapshot.json
│   │   │   │   ├── uigallery-todo.ir.json
│   │   │   │   └── uigallery-todo.snapshot.json
│   │   │   ├── support/
│   │   │   │   ├── jsonschema.rs
│   │   │   │   └── mod.rs
│   │   │   ├── code.rs
│   │   │   ├── golden.rs
│   │   │   ├── identity.rs
│   │   │   ├── prefer.rs
│   │   │   ├── properties.rs
│   │   │   ├── rules.rs
│   │   │   └── transform.rs
│   │   └── Cargo.toml
│   ├── record/
│   │   ├── src/
│   │   │   ├── scene/
│   │   │   │   ├── tests/
│   │   │   │   │   ├── mask.rs
│   │   │   │   │   └── mod.rs
│   │   │   │   ├── asset.rs
│   │   │   │   ├── bytes.rs
│   │   │   │   ├── mask.rs
│   │   │   │   ├── mirror.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── node.rs
│   │   │   │   └── send.rs
│   │   │   ├── visuals/
│   │   │   │   ├── mod.rs
│   │   │   │   └── reader.rs
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
│   │   │   ├── session.rs
│   │   │   ├── state.rs
│   │   │   ├── track.rs
│   │   │   └── weigh.rs
│   │   ├── tests/
│   │   │   ├── session/
│   │   │   │   ├── bound.rs
│   │   │   │   ├── file.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── readback.rs
│   │   │   │   ├── ring.rs
│   │   │   │   ├── secret.rs
│   │   │   │   ├── semantic.rs
│   │   │   │   ├── thesis.rs
│   │   │   │   ├── toys.rs
│   │   │   │   └── visual.rs
│   │   │   └── panic.rs
│   │   ├── build.rs
│   │   └── Cargo.toml
│   ├── replay/
│   │   ├── src/
│   │   │   ├── bin/
│   │   │   │   └── plev_inspect.rs
│   │   │   ├── prose/
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
│   │   │   └── queries/
│   │   │       ├── cli.rs
│   │   │       ├── control.rs
│   │   │       ├── export.rs
│   │   │       ├── main.rs
│   │   │       ├── prose.rs
│   │   │       ├── readonly.rs
│   │   │       ├── thesis.rs
│   │   │       ├── timeline.rs
│   │   │       ├── toys.rs
│   │   │       └── why.rs
│   │   └── Cargo.toml
│   ├── screen/
│   │   ├── src/
│   │   │   ├── host/
│   │   │   │   ├── mod.rs
│   │   │   │   └── prefer.rs
│   │   │   ├── code.rs
│   │   │   ├── draw.rs
│   │   │   ├── expand.rs
│   │   │   ├── layout.rs
│   │   │   ├── lib.rs
│   │   │   ├── project.rs
│   │   │   ├── surface.rs
│   │   │   ├── token.rs
│   │   │   ├── transition.rs
│   │   │   └── widget.rs
│   │   ├── tests/
│   │   │   ├── fixtures/
│   │   │   │   ├── mixer-left.ir.json
│   │   │   │   ├── mixer-right.ir.json
│   │   │   │   ├── uigallery-silences-compare.ir.json
│   │   │   │   ├── uigallery-silences-review.ir.json
│   │   │   │   └── uigallery-todo-columns.ir.json
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
│   │   │   ├── policy.rs
│   │   │   ├── prefer.rs
│   │   │   ├── promote.rs
│   │   │   ├── render.rs
│   │   │   ├── transform.rs
│   │   │   └── transition.rs
│   │   └── Cargo.toml
│   └── socket/
│       ├── src/
│       │   ├── bin/
│       │   │   └── plev.rs
│       │   ├── framing/
│       │   │   ├── base64.rs
│       │   │   └── mod.rs
│       │   ├── scene/
│       │   │   ├── mod.rs
│       │   │   └── read.rs
│       │   ├── client.rs
│       │   ├── error.rs
│       │   ├── lib.rs
│       │   ├── place.rs
│       │   ├── port.rs
│       │   ├── protocol.rs
│       │   ├── ring.rs
│       │   ├── server.rs
│       │   ├── shown.rs
│       │   ├── sink.rs
│       │   └── traffic.rs
│       ├── tests/
│       │   ├── support/
│       │   │   ├── mod.rs
│       │   │   └── toy.rs
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
├── tool/
│   ├── bench/
│   │   ├── measure/
│   │   │   ├── src/
│   │   │   │   ├── harness.rs
│   │   │   │   ├── main.rs
│   │   │   │   ├── probe_cost.rs
│   │   │   │   ├── report.rs
│   │   │   │   └── scenarios.rs
│   │   │   └── Cargo.toml
│   │   └── regress/
│   │       ├── baselines/
│   │       │   ├── accordion@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── accordion@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── alert-dialog@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── alert-dialog@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── alert@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── alert@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── aspect-ratio@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── aspect-ratio@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── attachment@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── attachment@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── avatar@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── avatar@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── badge@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── badge@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── breadcrumb@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── breadcrumb@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── bubble@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── bubble@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── builder@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── builder@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── button-group@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── button-group@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── button@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── button@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── calendar@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── calendar@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── capture@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── capture@hoff@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── card@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── card@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── carousel@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── carousel@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── chart@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── chart@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── checkbox@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── checkbox@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── chip@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── chip@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── code-block@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── code-block@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── collapsible@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── collapsible@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── combobox@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── combobox@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── command@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── command@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── context-menu@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── context-menu@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── data-table@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── data-table@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── date-picker@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── date-picker@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── dialog@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── dialog@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── dock@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── dock@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── drawer@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── drawer@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── dropdown-menu@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── dropdown-menu@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── editor@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── editor@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── effects@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── effects@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── empty@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── empty@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── field@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── field@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── files@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── files@hoff@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── git@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── git@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── graph@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── graph@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── hover-card@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── hover-card@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── icon-button@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── icon-button@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── icons@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── icons@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── input-group@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── input-group@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── input-otp@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── input-otp@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── input@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── input@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── item@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── item@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── kbd@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── kbd@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── label@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── label@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── marker@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── marker@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── menubar@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── menubar@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── message-scroller@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── message-scroller@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── message@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── message@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── navigation-menu@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── navigation-menu@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── pagination@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── pagination@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── panel-header@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── panel-header@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── panel@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── panel@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── popover@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── popover@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── progress@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── progress@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── questionnaire@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── questionnaire@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── radio-group@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── radio-group@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── resizable@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── resizable@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── scroll-area@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── scroll-area@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── select@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── select@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── separator@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── separator@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── sheet@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── sheet@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── sidebar@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── sidebar@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── silences-review-by-the-claude-fixture-adapted@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-claude-fixture-too-much-in-portuguese@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-claude-fixture@core@1200x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-claude-fixture@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-claude-fixture@hoff@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules-a-sequence-repeated@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules-can-t-compare@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules-larger-text-and-more-contrast@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules-the-sequence-as-one-action@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules-too-much-information@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules@core@1200x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences-review-by-the-rules@hoff@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── silences@hoff@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── skeleton@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── skeleton@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── slider@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── slider@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── spinner@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── spinner@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── stat@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── stat@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── switch@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── switch@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── table@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── table@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── tabs@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── tabs@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── textarea@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── textarea@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── theme@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── theme@hoff@1280x800/
│   │       │   │   └── scene.json
│   │       │   ├── toast@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── toast@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── todo@core@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── todo@core@390x844/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── todo@hoff@1280x800/
│   │       │   │   ├── frame.png
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── toggle-group@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── toggle-group@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── toggle@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── toggle@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── tooltip@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── tooltip@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── tree@core@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── tree@hoff@1280x800/
│   │       │   │   ├── scene.json
│   │       │   │   └── semantics.json
│   │       │   ├── typography@core@1280x800/
│   │       │   │   └── scene.json
│   │       │   └── typography@hoff@1280x800/
│   │       │       └── scene.json
│   │       ├── src/
│   │       │   ├── capture/
│   │       │   │   ├── mod.rs
│   │       │   │   ├── summary.rs
│   │       │   │   └── tests.rs
│   │       │   ├── report/
│   │       │   │   ├── describe.rs
│   │       │   │   ├── json.rs
│   │       │   │   ├── mod.rs
│   │       │   │   └── text.rs
│   │       │   ├── baseline.rs
│   │       │   ├── classify.rs
│   │       │   ├── compare.rs
│   │       │   ├── determinism.rs
│   │       │   ├── hash.rs
│   │       │   ├── main.rs
│   │       │   ├── pixels.rs
│   │       │   ├── requested.rs
│   │       │   ├── run.rs
│   │       │   ├── scene.rs
│   │       │   ├── state.rs
│   │       │   └── tests.rs
│   │       ├── Cargo.toml
│   │       └── requested.json
│   ├── tasks/
│   │   ├── depscheck.sh
│   │   ├── figmapull.py
│   │   ├── fullcheck.sh
│   │   ├── mobileapp.sh
│   │   ├── namecheck.sh
│   │   ├── precommit
│   │   ├── screencap.sh
│   │   ├── subsetter.py
│   │   └── webtarget.sh
│   └── tests/
│       ├── Cargo.toml
│       ├── test_depsgraph.rs
│       ├── test_filepaths.rs
│       ├── test_namerules.rs
│       ├── test_repoaudit.rs
│       └── test_structure.rs
├── .editorconfig
├── .gitattributes
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── rust-toolchain.toml
└── rustfmt.toml
```



&nbsp;

Utilizar as ferramentas e tools para agiliar o processo, testar autioar e fazer rapido com qualidade, less but better:  
  
  
Ferramentas, por etapa

- **Mapa:** um arquivo `migration.toml` com uma linha por arquivo: origem → destino → tipo (move, merge, split, new, drop). É a fonte única da migração; tudo o mais lê dele.
- **Mover:** `git mv` comandado pelo mapa, num `cargo xtask migrate`. Preserva o histórico; no macOS, a troca só de maiúscula passa por um nome temporário.  
[https://git-scm.com/docs/git-mv](https://git-scm.com/docs/git-mv)  
[https://github.com/matklad/cargo-xtask](https://github.com/matklad/cargo-xtask)
- **Commit só de moves, separado do commit de edições:** sem isso, o `git diff -M` perde os renomes abaixo de 50% de similaridade, como `mod.rs` pequenos e arquivos com imports reescritos. Com moves puros, tudo sai `R100`, e o `git log --follow` e o `blame` continuam íntegros.
- **Cargo.toml:** `toml_edit`. Reescreve `members`, `path =` e `[workspace.dependencies]` sem estragar a formatação.  
[https://github.com/toml-rs/toml](https://github.com/toml-rs/toml)
- **Renomear pacotes:** `cargo-workspaces` (`cargo ws rename`). Troca o nome em todos os manifests.  
[https://github.com/pksunkara/cargo-workspaces](https://github.com/pksunkara/cargo-workspaces)
- **Código Rust:** `ast-grep` (`ast-grep run -p 'old::$$$A' -r 'new::$$$A' -l rust -U`). Reescreve caminhos de forma sintática, não por texto.  
[https://github.com/ast-grep/ast-grep](https://github.com/ast-grep/ast-grep)
- **Módulos e símbolos:** rust-analyzer via LSP ("rename"). Renomeia o módulo e atualiza todos os usos, ciente de tipos.  
[https://github.com/rust-lang/rust-analyzer](https://github.com/rust-lang/rust-analyzer)
- **Caminhos em texto (CI, scripts, docs,** `.gitignore` **e também** `.rs`**):** `fastmod --accept-all` e `sd`. Troca em massa de caminhos.  
O `ast-grep` reescreve caminhos de módulo, mas não literais. Ficam de fora dele:
  - `include_bytes!`, `include_str!`;
  - `concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/...")`;
  - `#[path = ...]`;
  - fakes de teste (`tests/fake-ffmpeg` → `tests/fakes/ffmpeg`).

  Por isso o `fastmod`/`sd` roda também em `.rs`, não só fora do Rust. **Atenção à profundidade:** quando um crate muda de nível (`tool/bench/regress` fica um nível abaixo de `crates/showcase/examples/regress`), os `../` dos caminhos relativos mudam.  
  [https://github.com/facebookincubator/fastmod](https://github.com/facebookincubator/fastmod)  
  [https://github.com/chmln/sd](https://github.com/chmln/sd)
- **Achar o que sobrou:** `ripgrep`. A busca pelos caminhos antigos precisa voltar vazia fora do histórico.  
[https://github.com/BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)
- **Fusão de crates sem quebrar:** shim temporário. O crate antigo vira só `pub use novo::modulo::*;`, compila na hora, e o `ast-grep` troca os usos depois, com calma.  
É uma técnica, sem ferramenta própria.

## A prova de que nada quebrou

**As comparações "mesma lista", "mesma API" e "mesmo metadata" divergem por design.** Crates são fundidos e renomeados (`comps` → `engine::widgets`, `rope` → `editor`, `lot` + `svg` + `parser` → `import`), e módulos mudam de lugar (`engine::theme` → `engine::styling::theme`, `tests_donut` → `tests::donut`). A comparação só vale depois de normalizar os nomes pelo `migration.toml`: o lado "antes" é traduzido para os nomes novos e só então comparado com o "depois".

- **Mesma lista de testes, todos passando:** `cargo nextest list --message-format json` antes e depois, com os nomes normalizados pelo mapa, mais `cargo nextest run`.  
[https://github.com/nextest-rs/nextest](https://github.com/nextest-rs/nextest)
- **Mesma API pública das libs:** `cargo semver-checks --baseline-rev main` só para os crates que mantêm o nome; ele não casa um crate renomeado com o baseline. Para os renomeados e fundidos, `cargo public-api diff` com os caminhos normalizados.  
[https://github.com/obi1kenobi/cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)  
[https://github.com/cargo-public-api/cargo-public-api](https://github.com/cargo-public-api/cargo-public-api)
- **Mesmo comportamento:** testes de snapshot com saídas idênticas antes e depois (cenas, árvores semânticas, textos gerados). O repositório ainda não tem nenhum, então **os snapshots são criados e gravados antes do primeiro move**. Sem esse "antes", não há com o que comparar.  
[https://github.com/mitsuhiko/insta](https://github.com/mitsuhiko/insta)
- **Mesmos pacotes e dependências:** `cargo metadata` e `cargo tree` comparados, com os nomes de pacote normalizados pelo mapa.  
[https://doc.rust-lang.org/cargo/commands/cargo-metadata.html](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)  
[https://doc.rust-lang.org/cargo/commands/cargo-tree.html](https://doc.rust-lang.org/cargo/commands/cargo-tree.html)
- **A estrutura bate com o alvo:** testes de arquitetura (nomes, dependências, raiz) sobre `git ls-files`, mais um `diff` com a árvore-alvo.  
[https://github.com/oli-obk/cargo_metadata](https://github.com/oli-obk/cargo_metadata)
- **Nada se perdeu:** o mapa conferido nos dois sentidos (todo arquivo de hoje tem destino, todo arquivo do alvo tem origem ou é novo), mais `git diff -M --name-status` rodado **sobre o commit só de moves** (`git diff -M --name-status <commit>^ <commit>`), onde cada move aparece como `R`, renome. Não rode sobre o range inteiro (`main..HEAD`): ali, os arquivos com imports reescritos podem cair abaixo de 50% de similaridade e aparecer como `D` + `A`.  
[https://git-scm.com/docs/git-diff](https://git-scm.com/docs/git-diff)
- **Docs sem link quebrado:** `lychee`.  
[https://github.com/lycheeverse/lychee](https://github.com/lycheeverse/lychee)
- **Compila em todo alvo:** `cargo check --target <alvo>` para cada plataforma, e as builds nativas (Android com `cargo-ndk`, iOS com `xcodebuild`).  
[https://rust-lang.github.io/rustup/cross-compilation.html](https://rust-lang.github.io/rustup/cross-compilation.html)  
[https://github.com/bbqsrc/cargo-ndk](https://github.com/bbqsrc/cargo-ndk)



&nbsp;

&nbsp;

agora, crie uma lista de tarefas completa com todos itens da tarefa,

  uma por linha e execute um por inha, se aparecer alhgum impevisto,

  mudança adicone ao plano e execute, pois o plano devera ser adutado pelo

  tech lead, dev e ctp que vao configurar sua lista de tarefas e execuçao entregada.