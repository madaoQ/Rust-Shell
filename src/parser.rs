//! parser.rs

// 辅助结构体，用于存储解析后的命令信息
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    pub name: String,
    pub args: Vec<String>,
    pub stdin_redirect: Option<String>,
    pub stdout_redirect: Option<(String, bool)>, // (文件名, 是否为追加模式)
    pub stderr_redirect: Option<String>,         // (文件名) 对于 2>
}

/// 将单个命令行字符串（不含管道）解析为 ParsedCommand 结构体。
/// 这个解析器是基础版本：它处理以空格分隔的参数和简单的 I/O 重定向（<, >, >>, 2>）。
/// 它不处理以下情况：
///   - 带引号的参数（例如, "hello world"）
///   - 转义字符
///   - 命令替换 (`$()`) 了
///   - 后台进程 (`&`)
pub fn parse_single_command(command_segment: &str) -> Result<ParsedCommand, String> {
    let parts = command_segment.split_whitespace().collect::<Vec<&str>>();
    if parts.is_empty() {
        return Err("空命令段".to_string());
    }

    let name = parts[0].to_string();
    let mut args = Vec::new();
    let mut stdin_redirect: Option<String> = None;
    let mut stdout_redirect: Option<(String, bool)> = None;
    let mut stderr_redirect: Option<String> = None;

    let mut i = 1; // 从第二个部分开始处理
    while i < parts.len() {
        match parts[i] {
            "<" => {
                if i + 1 < parts.len() {
                    stdin_redirect = Some(parts[i+1].to_string());
                    i += 2; // 跳过操作符和文件名
                } else {
                    return Err("输入重定向缺少文件名 (<)".to_string());
                }
            },
            ">" => {
                if i + 1 < parts.len() {
                    stdout_redirect = Some((parts[i+1].to_string(), false)); // false 表示覆盖模式
                    i += 2; // 跳过操作符和文件名
                } else {
                    return Err("输出重定向缺少文件名 (>)\nmy_shell: 解析错误:".to_string());
                }
            },
            ">>" => {
                if i + 1 < parts.len() {
                    stdout_redirect = Some((parts[i+1].to_string(), true)); // true 表示追加模式
                    i += 2; // 跳过操作符和文件名
                } else {
                    return Err("输出重定向缺少文件名 (>>)".to_string());
                }
            },
            "2>" => {
                if i + 1 < parts.len() {
                    stderr_redirect = Some(parts[i+1].to_string());
                    i += 2; // 跳过操作符和文件名
                } else {
                    return Err("标准错误重定向缺少文件名 (2>)".to_string());
                }
            },
            _ => {
                // 如果不是重定向操作符，则将其作为参数
                args.push(parts[i].to_string());
                i += 1;
            }
        }
    }

    Ok(ParsedCommand {
        name,
        args,
        stdin_redirect,
        stdout_redirect,
        stderr_redirect,
    })
}

/// 解析包含管道符的完整命令行。
/// 将命令行分割成多个命令段，并为每个命令段调用 parse_single_command。
pub fn parse_pipeline_commands(command_line: &str) -> Result<Vec<ParsedCommand>, String> {
    let segments: Vec<&str> = command_line.split('|').collect();
    let mut commands = Vec::new();

    for segment in segments {
        if segment.trim().is_empty() {
            return Err("管道符 ' | ' 后不能有空命令.".to_string());
        }
        commands.push(parse_single_command(segment.trim())?);
    }
    Ok(commands)
} 