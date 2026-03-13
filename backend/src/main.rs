// 在 Windows 发布版本中隐藏控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod git_history;

use git_history::{get_git_history_impl, rewrite_git_history_impl, RewriteRequest};

#[tauri::command]
fn get_git_history(app: tauri::AppHandle, repo_path: String) -> Result<Vec<git_history::GitCommit>, String> {
    get_git_history_impl(&app, &repo_path)
}

#[tauri::command]
fn rewrite_git_history(app: tauri::AppHandle, request: RewriteRequest) -> Result<git_history::RewriteResult, String> {
    rewrite_git_history_impl(&app, request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_git_history, rewrite_git_history])
        .run(tauri::generate_context!())
        .expect("tauri app 运行失败");
}

fn main() {
    run();
}
