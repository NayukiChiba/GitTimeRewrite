// 在 Windows 发布版本中隐藏控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod git_history;

use git_history::{
    force_push_origin_impl, get_git_history_impl, get_git_origin_impl, rewrite_git_history_impl,
    set_git_origin_impl, RewriteRequest,
};

#[tauri::command]
fn get_git_history(app: tauri::AppHandle, repo_path: String) -> Result<Vec<git_history::GitCommit>, String> {
    get_git_history_impl(&app, &repo_path)
}

#[tauri::command]
async fn rewrite_git_history(
    app: tauri::AppHandle,
    request: RewriteRequest,
) -> Result<git_history::RewriteResult, String> {
    tauri::async_runtime::spawn_blocking(move || rewrite_git_history_impl(&app, request))
        .await
        .map_err(|error| format!("改写任务执行失败: {error}"))?
}

#[tauri::command]
fn set_git_origin(app: tauri::AppHandle, repo_path: String, origin_url: String) -> Result<String, String> {
    set_git_origin_impl(&app, &repo_path, &origin_url)
}

#[tauri::command]
fn force_push_origin(app: tauri::AppHandle, repo_path: String) -> Result<String, String> {
    force_push_origin_impl(&app, &repo_path)
}

#[tauri::command]
fn get_git_origin(app: tauri::AppHandle, repo_path: String) -> Result<String, String> {
    get_git_origin_impl(&app, &repo_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_git_history,
            rewrite_git_history,
            set_git_origin,
            force_push_origin,
            get_git_origin
        ])
        .run(tauri::generate_context!())
        .expect("tauri app 运行失败");
}

fn main() {
    run();
}
