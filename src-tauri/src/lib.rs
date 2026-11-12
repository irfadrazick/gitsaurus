pub mod commands;
pub mod git;
pub mod state;
pub mod system;
pub mod watcher;

use tauri::menu::Menu;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(state::AppState::default())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Resolve the login-shell PATH/SSH_AUTH_SOCK off the launch path; the first git
            // call waits on this (OnceLock) instead of spawning the shell itself.
            std::thread::spawn(|| {
                git::git_env();
            });
            // Default app menu so Cmd+C/V/A work in text inputs.
            let menu = Menu::default(app.handle())?;
            app.set_menu(menu)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::open_repo,
            commands::load_graph,
            commands::graph_page,
            commands::find_commit,
            commands::search_commits,
            commands::list_refs,
            commands::repo_state,
            commands::commit_details,
            commands::file_diff,
            commands::last_commit_message,
            commands::stage,
            commands::stage_all,
            commands::unstage,
            commands::unstage_all,
            commands::discard,
            commands::commit,
            commands::checkout,
            commands::checkout_commit,
            commands::create_branch,
            commands::rename_branch,
            commands::delete_branch,
            commands::validate_branch_name,
            commands::validate_tag_name,
            commands::create_tag,
            commands::delete_tag,
            commands::push_tag,
            commands::reset,
            commands::stash_push,
            commands::stash_apply,
            commands::stash_pop,
            commands::stash_drop,
            commands::merge,
            commands::rebase,
            commands::cherry_pick,
            commands::revert,
            commands::continue_op,
            commands::skip_op,
            commands::abort_op,
            commands::resolve_conflict,
            commands::mark_resolved,
            commands::fetch_all,
            commands::pull,
            commands::pull_branch,
            commands::push,
            commands::set_upstream,
            commands::cancel_network,
            commands::watch_repo,
            commands::unwatch_repo,
            commands::open_in_editor,
            commands::remove_worktree,
            commands::prune_worktrees,
            commands::reveal_in_finder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
