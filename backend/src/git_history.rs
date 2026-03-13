use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tauri::{AppHandle, Emitter};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitCommit {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitEdit {
    pub id: String,
    pub message: Option<String>,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub author_date: Option<String>,
    pub committer_name: Option<String>,
    pub committer_email: Option<String>,
    pub committer_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteRequest {
    pub repo_path: String,
    pub edits: Vec<CommitEdit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteResult {
    pub rewritten_count: usize,
    pub output: String,
}

pub fn get_git_history_impl(app: &AppHandle, repo_path: &str) -> Result<Vec<GitCommit>, String> {
    emit_log(app, &format!("正在验证 Git 仓库: {repo_path}"));
    ensure_git_repo(repo_path)?;

    emit_log(app, "正在读取提交历史...");
    let format = "%H%x1f%P%x1f%an%x1f%ae%x1f%aI%x1f%cn%x1f%ce%x1f%cI%x1f%s%x1e";
    let output = run_git_command(repo_path, &["log", "--reverse", "--date=iso-strict", &format!("--pretty=format:{format}")])?;

    let commits = parse_git_log_output(&output);
    emit_log(app, &format!("读取完成，共 {} 条提交", commits.len()));
    Ok(commits)
}

pub fn rewrite_git_history_impl(app: &AppHandle, request: RewriteRequest) -> Result<RewriteResult, String> {
    emit_log(app, &format!("正在验证 Git 仓库: {}", request.repo_path));
    ensure_git_repo(&request.repo_path)?;

    if request.edits.is_empty() {
        return Err("没有任何需要改写的提交".to_string());
    }

    emit_log(app, &format!("准备改写 {} 条提交，构建过滤脚本...", request.edits.len()));
    let env_filter = build_env_filter_script(&request.edits);
    let msg_filter = build_msg_filter_script(&request.edits);

    emit_log(app, "正在执行 git filter-branch，请稍候...");
    let output = run_git_command(
        &request.repo_path,
        &[
            "filter-branch",
            "-f",
            "--env-filter",
            &env_filter,
            "--msg-filter",
            &msg_filter,
            "--tag-name-filter",
            "cat",
            "--",
            "--all",
        ],
    )?;

    emit_log(app, &format!("改写完成，共处理 {} 条提交", request.edits.len()));
    Ok(RewriteResult {
        rewritten_count: request.edits.len(),
        output,
    })
}

pub fn set_git_origin_impl(app: &AppHandle, repo_path: &str, origin_url: &str) -> Result<String, String> {
    emit_log(app, &format!("正在验证 Git 仓库: {repo_path}"));
    ensure_git_repo(repo_path)?;

    let trimmed_url = origin_url.trim();
    if trimmed_url.is_empty() {
        return Err("origin 地址不能为空".to_string());
    }

    let has_origin = run_git_command(repo_path, &["remote", "get-url", "origin"]).is_ok();
    if has_origin {
        emit_log(app, "检测到已存在 origin，正在更新地址...");
        run_git_command(repo_path, &["remote", "set-url", "origin", trimmed_url])?;
    } else {
        emit_log(app, "未检测到 origin，正在新增远程仓库...");
        run_git_command(repo_path, &["remote", "add", "origin", trimmed_url])?;
    }

    emit_log(app, "origin 设置完成");
    Ok(trimmed_url.to_string())
}

pub fn force_push_origin_impl(app: &AppHandle, repo_path: &str) -> Result<String, String> {
    emit_log(app, &format!("正在验证 Git 仓库: {repo_path}"));
    ensure_git_repo(repo_path)?;

    emit_log(app, "正在执行 git push --force origin HEAD...");
    let output = run_git_command(repo_path, &["push", "--force", "origin", "HEAD"])?;
    emit_log(app, "force push 已完成");
    Ok(output)
}

fn emit_log(app: &AppHandle, message: &str) {
    app.emit("log", message).ok();
}

fn ensure_git_repo(repo_path: &str) -> Result<(), String> {
    let output = run_git_command(repo_path, &["rev-parse", "--is-inside-work-tree"])?;
    if output.trim() != "true" {
        return Err(format!("路径不是有效 Git 仓库: {repo_path}"));
    }
    Ok(())
}

fn run_git_command(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_path).args(args);

    #[cfg(target_os = "windows")]
    {
        // CREATE_NO_WINDOW，避免执行 git 时弹出控制台窗口
        command.creation_flags(0x08000000);
    }

    let result = command
        .output()
        .map_err(|e| format!("执行 git 失败: {e}"))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);
        return Err(format!(
            "git 命令执行失败: git {}\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            stdout,
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&result.stdout).to_string())
}

fn parse_git_log_output(raw: &str) -> Vec<GitCommit> {
    raw.split('\x1e')
        .filter_map(|record| {
            let trimmed = record.trim();
            if trimmed.is_empty() {
                return None;
            }

            let fields: Vec<&str> = trimmed.split('\x1f').collect();
            if fields.len() != 9 {
                return None;
            }

            Some(GitCommit {
                id: fields[0].to_string(),
                parent_ids: fields[1]
                    .split_whitespace()
                    .map(|parent| parent.to_string())
                    .collect(),
                author_name: fields[2].to_string(),
                author_email: fields[3].to_string(),
                author_date: fields[4].to_string(),
                committer_name: fields[5].to_string(),
                committer_email: fields[6].to_string(),
                committer_date: fields[7].to_string(),
                message: fields[8].to_string(),
            })
        })
        .collect()
}

fn build_env_filter_script(edits: &[CommitEdit]) -> String {
    let mut lines = vec!["case \"$GIT_COMMIT\" in".to_string()];

    for edit in edits {
        let mut edit_lines = Vec::new();

        if let Some(author_name) = &edit.author_name {
            edit_lines.push(format!(
                "export GIT_AUTHOR_NAME='{}'",
                escape_for_shell_single_quoted(author_name)
            ));
        }
        if let Some(author_email) = &edit.author_email {
            edit_lines.push(format!(
                "export GIT_AUTHOR_EMAIL='{}'",
                escape_for_shell_single_quoted(author_email)
            ));
        }
        if let Some(author_date) = &edit.author_date {
            edit_lines.push(format!(
                "export GIT_AUTHOR_DATE='{}'",
                escape_for_shell_single_quoted(author_date)
            ));
        }
        if let Some(committer_name) = &edit.committer_name {
            edit_lines.push(format!(
                "export GIT_COMMITTER_NAME='{}'",
                escape_for_shell_single_quoted(committer_name)
            ));
        }
        if let Some(committer_email) = &edit.committer_email {
            edit_lines.push(format!(
                "export GIT_COMMITTER_EMAIL='{}'",
                escape_for_shell_single_quoted(committer_email)
            ));
        }
        if let Some(committer_date) = &edit.committer_date {
            edit_lines.push(format!(
                "export GIT_COMMITTER_DATE='{}'",
                escape_for_shell_single_quoted(committer_date)
            ));
        }

        if !edit_lines.is_empty() {
            lines.push(format!("{} )", edit.id));
            lines.extend(edit_lines);
            lines.push(";;".to_string());
        }
    }

    lines.push("* )".to_string());
    lines.push(";;".to_string());
    lines.push("esac".to_string());

    lines.join("\n")
}

fn build_msg_filter_script(edits: &[CommitEdit]) -> String {
    let mut msg_map: HashMap<&str, &str> = HashMap::new();
    for edit in edits {
        if let Some(message) = &edit.message {
            msg_map.insert(edit.id.as_str(), message.as_str());
        }
    }

    if msg_map.is_empty() {
        return "cat".to_string();
    }

    let mut lines = vec!["case \"$GIT_COMMIT\" in".to_string()];
    for (commit_id, message) in msg_map {
        lines.push(format!("{} )", commit_id));
        lines.push(format!("printf '%s\\n' '{}'", escape_for_shell_single_quoted(message)));
        lines.push(";;".to_string());
    }
    lines.push("* ) cat ;;".to_string());
    lines.push("esac".to_string());

    lines.join("\n")
}

fn escape_for_shell_single_quoted(value: &str) -> String {
    value.replace('\'', "'\"'\"'")
}

#[cfg(test)]
mod tests {
    use super::{build_env_filter_script, build_msg_filter_script, parse_git_log_output, CommitEdit};

    #[test]
    fn should_parse_git_log_output() {
        let raw = "a1\x1f\x1fN\x1fn@example.com\x1f2026-01-01T01:01:01+08:00\x1fN\x1fn@example.com\x1f2026-01-01T01:01:01+08:00\x1finit\x1e";
        let commits = parse_git_log_output(raw);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].id, "a1");
        assert_eq!(commits[0].message, "init");
    }

    #[test]
    fn should_build_env_filter_script() {
        let edits = vec![CommitEdit {
            id: "abc".to_string(),
            message: None,
            author_name: Some("A".to_string()),
            author_email: None,
            author_date: Some("2026-01-01T10:00:00+08:00".to_string()),
            committer_name: None,
            committer_email: None,
            committer_date: None,
        }];

        let script = build_env_filter_script(&edits);
        assert!(script.contains("abc )"));
        assert!(script.contains("GIT_AUTHOR_NAME='A'"));
    }

    #[test]
    fn should_build_msg_filter_script() {
        let edits = vec![CommitEdit {
            id: "abc".to_string(),
            message: Some("hello".to_string()),
            author_name: None,
            author_email: None,
            author_date: None,
            committer_name: None,
            committer_email: None,
            committer_date: None,
        }];

        let script = build_msg_filter_script(&edits);
        assert!(script.contains("abc )"));
        assert!(script.contains("hello"));
    }
}
