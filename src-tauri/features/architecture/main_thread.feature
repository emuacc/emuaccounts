Feature: The main thread only runs the window

  The window's event loop runs on the main thread. Any other work there stops
  it painting and taking clicks, and the OS reports the app as Not Responding:
  a large vault's launch rebuild once held it for over 7 minutes, and the
  issues scan for over a minute more.

  So opening a data folder, at launch or by switching to another one, hands
  its rebuild to a background thread while the window shows "Rebuilding
  cache". Tauri dispatches a plain `#[tauri::command] fn` on the main thread
  and an `async fn` or a `#[tauri::command(async)]` off it, so every command
  is one of those two. The one exception is frontend_log, a single eprintln:
  the UI fires it without awaiting, and only the main thread's in-order
  dispatch keeps the terminal log in the order the UI wrote it.

  Scenario: Only frontend_log is dispatched on the main thread
    Given the app's Rust sources
    Then no Tauri command except "frontend_log" is dispatched on the main thread

  Scenario: The setup hook hands the vault rebuild to a background thread
    Given the app's Rust sources
    Then the setup hook calls "spawn_vault_rebuild"
    And the setup hook does not call "run_pipeline"
    And the setup hook does not call "rebuild_vault"
