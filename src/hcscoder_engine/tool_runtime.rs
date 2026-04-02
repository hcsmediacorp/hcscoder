//! Maps the public tool belt (`hcscoder_tools::get_all_tools`) to runnable implementations.
//! Used by the engine and tests to ensure every registered name is dispatchable.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::hcscoder_tools;

fn take_str<'a>(args: &'a Value, key: &str) -> Result<&'a str> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing or invalid string field {:?}", key))
}

fn take_str_opt<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

fn default_repo(args: &Value) -> &str {
    take_str_opt(args, "repo").unwrap_or(".")
}

/// Tool names from the registry (same order as `get_all_tools`).
pub fn registered_tool_names() -> Vec<&'static str> {
    hcscoder_tools::get_all_tools()
        .into_iter()
        .map(|t| t.name)
        .collect()
}

fn json_pretty<T: Serialize>(v: &T) -> Result<String> {
    serde_json::to_string_pretty(v).context("serialize tool output")
}

fn out_debug<T: std::fmt::Debug>(v: &T) -> String {
    format!("{:#?}", v)
}

/// Invoke a registered tool by name. `args` is a JSON object; required fields vary by tool.
pub async fn invoke_tool(name: &str, args: &Value) -> Result<String> {
    match name {
        "bash" => {
            let cmd = take_str(args, "command")?;
            let out = hcscoder_tools::bash::execute_command(cmd).await?;
            Ok(format!(
                "exit={}\n--- stdout ---\n{}\n--- stderr ---\n{}",
                out.exit_code, out.stdout, out.stderr
            ))
        }
        "bash_background" => {
            let cmd = take_str(args, "command")?;
            let pid = hcscoder_tools::bash::execute_background(cmd).await?;
            Ok(format!("background pid: {}", pid))
        }
        "read_file" => {
            let path = take_str(args, "path")?;
            hcscoder_tools::filesystem::read_file(path).await
        }
        "write_file" => {
            let path = take_str(args, "path")?;
            let content = take_str(args, "content")?;
            hcscoder_tools::filesystem::write_file(path, content)
                .await
                .map(|_| format!("wrote {}", path))
        }
        "list_directory" => {
            let path = take_str(args, "path")?;
            let entries = hcscoder_tools::filesystem::list_directory(path).await?;
            Ok(out_debug(&entries))
        }
        "search_files" => {
            let root = take_str(args, "root")?;
            let pattern = take_str(args, "pattern")?;
            let paths = hcscoder_tools::filesystem::search_files(root, pattern).await?;
            Ok(out_debug(&paths))
        }
        "create_directory" => {
            let path = take_str(args, "path")?;
            hcscoder_tools::filesystem::create_directory(path)
                .await
                .map(|_| format!("created {}", path))
        }
        "delete_file" => {
            let path = take_str(args, "path")?;
            hcscoder_tools::filesystem::delete_file(path)
                .await
                .map(|_| format!("deleted {}", path))
        }
        "delete_directory" => {
            let path = take_str(args, "path")?;
            hcscoder_tools::filesystem::delete_directory(path)
                .await
                .map(|_| format!("removed directory {}", path))
        }
        "move_file" => {
            let from = take_str(args, "from")?;
            let to = take_str(args, "to")?;
            hcscoder_tools::filesystem::move_path(from, to)
                .await
                .map(|_| format!("moved {} -> {}", from, to))
        }
        "copy_file" => {
            let from = take_str(args, "from")?;
            let to = take_str(args, "to")?;
            hcscoder_tools::filesystem::copy_file(from, to)
                .await
                .map(|_| format!("copied {} -> {}", from, to))
        }
        "append_file" => {
            let path = take_str(args, "path")?;
            let content = take_str(args, "content")?;
            hcscoder_tools::filesystem::append_file(path, content)
                .await
                .map(|_| format!("appended to {}", path))
        }
        "file_metadata" => {
            let path = take_str(args, "path")?;
            let m = hcscoder_tools::filesystem::get_metadata(path).await?;
            Ok(out_debug(&m))
        }
        "lsp_diagnostics" => {
            let path = take_str(args, "path")?;
            let lines = hcscoder_tools::lsp::get_diagnostics(path).await?;
            Ok(lines.join("\n"))
        }
        "lsp_definitions" => {
            let path = take_str(args, "path")?;
            let symbol = take_str(args, "symbol")?;
            let hits = hcscoder_tools::lsp::find_definitions(path, symbol).await?;
            Ok(hits.join("\n"))
        }
        "lsp_references" => {
            let path = take_str(args, "path")?;
            let symbol = take_str(args, "symbol")?;
            let hits = hcscoder_tools::lsp::find_references(path, symbol).await?;
            Ok(hits.join("\n"))
        }
        "lsp_hover" => {
            let path = take_str(args, "path")?;
            let symbol = take_str(args, "symbol")?;
            hcscoder_tools::lsp::hover_symbol(path, symbol).await
        }
        "lsp_completion" => {
            let path = take_str(args, "path")?;
            let prefix = take_str(args, "prefix")?;
            let v = hcscoder_tools::lsp::completion_prefix(path, prefix).await?;
            Ok(out_debug(&v))
        }
        "web_search" => {
            let query = take_str(args, "query")?;
            let results = hcscoder_tools::web::web_search(query).await?;
            Ok(out_debug(&results))
        }
        "fetch_url" => {
            let url = take_str(args, "url")?;
            hcscoder_tools::web::fetch_url(url).await
        }
        "git_status" => {
            let repo = default_repo(args);
            hcscoder_tools::git::git_status(repo).await
        }
        "git_diff" => {
            let repo = default_repo(args);
            hcscoder_tools::git::git_diff(repo).await
        }
        "git_log" => {
            let repo = default_repo(args);
            let n = args
                .get("max_entries")
                .and_then(|v| v.as_u64())
                .unwrap_or(20) as usize;
            hcscoder_tools::git::git_log(repo, n).await
        }
        "git_branch" => {
            let repo = default_repo(args);
            let list = args.get("list").and_then(|v| v.as_bool()).unwrap_or(false);
            if list {
                hcscoder_tools::git::git_branch_list(repo).await
            } else {
                hcscoder_tools::git::git_branch_show_current(repo).await
            }
        }
        "system_snapshot" => {
            let snap = hcscoder_tools::sys::system_snapshot()?;
            json_pretty(&snap)
        }
        "env_dump" => {
            let prefix = take_str_opt(args, "prefix");
            let vars = hcscoder_tools::sys::env_dump(prefix);
            Ok(vars
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("\n"))
        }
        "net_resolve" => {
            let host = take_str(args, "host")?;
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(443) as u16;
            let addrs = hcscoder_tools::net::resolve_host(host, port).await?;
            Ok(addrs
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join("\n"))
        }
        "tcp_probe" => {
            let host = take_str(args, "host")?;
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(443) as u16;
            let timeout_ms = args
                .get("timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(3000);
            let addrs = hcscoder_tools::net::resolve_host(host, port).await?;
            let first = addrs
                .first()
                .copied()
                .ok_or_else(|| anyhow!("no addresses"))?;
            hcscoder_tools::net::tcp_probe(first, timeout_ms).await?;
            Ok(format!("TCP ok to {}", first))
        }
        "think" => {
            let text = take_str_opt(args, "text")
                .or_else(|| take_str_opt(args, "topic"))
                .ok_or_else(|| anyhow!("need text or topic"))?;
            Ok(hcscoder_tools::utility::think(text))
        }
        "summarize" => {
            let text = take_str_opt(args, "text")
                .or_else(|| take_str_opt(args, "content"))
                .ok_or_else(|| anyhow!("need text or content"))?;
            Ok(hcscoder_tools::utility::summarize(text))
        }
        "explain" => {
            let s = take_str_opt(args, "subject")
                .or_else(|| take_str_opt(args, "text"))
                .ok_or_else(|| anyhow!("need subject or text"))?;
            Ok(hcscoder_tools::utility::explain(s))
        }
        "grep" => {
            let pattern = take_str(args, "pattern")?;
            let root = take_str(args, "root")?;
            let inc = take_str_opt(args, "include");
            let hits = hcscoder_tools::grep::grep(pattern, root, inc).await?;
            Ok(out_debug(&hits))
        }
        "grep_regex" => {
            let pattern = take_str(args, "pattern")?;
            let root = take_str(args, "root")?;
            let inc = take_str_opt(args, "include");
            let hits = hcscoder_tools::grep::grep_regex(pattern, root, inc).await?;
            Ok(out_debug(&hits))
        }
        "grep_count" => {
            let pattern = take_str(args, "pattern")?;
            let root = take_str(args, "root")?;
            let inc = take_str_opt(args, "include");
            let n = hcscoder_tools::grep::grep_count(pattern, root, inc).await?;
            Ok(format!("{}", n))
        }
        "glob" => {
            let pattern = take_str(args, "pattern")?;
            let root = take_str(args, "root")?;
            let entries = hcscoder_tools::glob::glob(pattern, root).await?;
            Ok(out_debug(&entries))
        }
        "apply_edit" => {
            let path = take_str(args, "path")?;
            let old = take_str(args, "old_string")?;
            let new = take_str(args, "new_string")?;
            hcscoder_tools::file_edit::apply_edit(path, old, new).await
        }
        "list_skills" => {
            let skills = hcscoder_tools::skill::list_skills().await?;
            Ok(out_debug(&skills))
        }
        "execute_skill" => {
            let id = take_str(args, "skill_id")?;
            let extra = args.get("skill_args").cloned().unwrap_or(Value::Null);
            hcscoder_tools::skill::execute_skill(id, extra).await
        }
        "create_task" => {
            let title = take_str(args, "title")?;
            let desc = take_str_opt(args, "description").map(|s| s.to_string());
            let t = hcscoder_tools::task::create_task(title.to_string(), desc).await?;
            Ok(out_debug(&t))
        }
        "list_tasks" => {
            let tasks = hcscoder_tools::task::list_tasks().await?;
            Ok(out_debug(&tasks))
        }
        "find_code_files" => {
            let root = take_str(args, "root")?;
            let files = hcscoder_tools::glob::find_code_files(root).await?;
            Ok(out_debug(&files))
        }
        _ => Err(anyhow!("unknown tool {:?}", name)),
    }
}

/// True if `name` is listed in `get_all_tools`.
pub fn is_registered_tool(name: &str) -> bool {
    hcscoder_tools::get_all_tools()
        .iter()
        .any(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn registry_count_40_plus() {
        assert!(registered_tool_names().len() >= 40);
    }

    #[tokio::test]
    async fn dispatch_think() {
        let out = invoke_tool("think", &json!({ "text": "step through it" }))
            .await
            .expect("think");
        assert!(out.contains("Thought"));
    }

    #[tokio::test]
    async fn dispatch_summarize() {
        let out = invoke_tool("summarize", &json!({ "text": "hello" }))
            .await
            .expect("summarize");
        assert!(out.contains("hello"));
    }

    #[tokio::test]
    async fn no_unknown_tool_for_registry_entries() {
        for t in hcscoder_tools::get_all_tools() {
            match invoke_tool(t.name, &json!({})).await {
                Ok(_) => {}
                Err(e) => assert!(!e.to_string().contains("unknown tool"), "{}: {}", t.name, e),
            }
        }
        let err = invoke_tool("not_a_real_tool_ever", &json!({}))
            .await
            .expect_err("bogus name");
        assert!(err.to_string().contains("unknown tool"));
    }
}
