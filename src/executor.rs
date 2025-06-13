use std::process::{Command, Stdio, Child};
use std::fs::File;
use crate::parser::ParsedCommand;

/// 执行一系列通过管道连接的命令。
/// 处理 I/O 重定向和管道的连接。
pub fn execute_pipeline(parsed_commands: &[ParsedCommand]) {
    let mut children: Vec<Child> = Vec::new();
    let mut previous_command_stdout: Option<std::process::ChildStdout> = None;

    for (i, parsed_cmd) in parsed_commands.iter().enumerate() {
        let mut command_builder = Command::new(&parsed_cmd.name);
        command_builder.args(&parsed_cmd.args);

        // 设置标准输入
        if let Some(prev_stdout) = previous_command_stdout.take() {
            // 如果是管道中的后续命令，则将前一个命令的输出作为当前命令的输入
            command_builder.stdin(prev_stdout);
        } else if i == 0 {
            // 如果是管道中的第一个命令，且有输入重定向
            if let Some(filepath) = &parsed_cmd.stdin_redirect {
                match File::open(filepath) {
                    Ok(file) => {
                        command_builder.stdin(Stdio::from(file));
                    },
                    Err(e) => {
                        eprintln!("my_shell: 无法打开输入文件 {}: {}", filepath, e);
                        // 如果输入文件无法打开，则清除之前启动的子进程，并中断管道
                        for c in children.iter_mut() {
                            let _ = c.kill(); 
                        }
                        children.clear();
                        break; 
                    }
                }
            }
        }

        // 设置标准输出
        // 如果不是管道中的最后一个命令，则管道输出到下一个命令
        if i < parsed_commands.len() -1 {
            command_builder.stdout(Stdio::piped());
        } else if let Some((filepath, append)) = &parsed_cmd.stdout_redirect {
            // 如果是管道中的最后一个命令，且有输出重定向
            let file_result = if *append {
                File::options().create(true).append(true).open(filepath)
            } else {
                File::create(filepath)
            };
            match file_result {
                Ok(file) => {
                    command_builder.stdout(Stdio::from(file));
                },
                Err(e) => {
                    eprintln!("my_shell: 无法打开输出文件 {}: {}", filepath, e);
                    // 如果输出文件无法打开，则清除之前启动的子进程，并中断管道
                    for c in children.iter_mut() {
                        let _ = c.kill(); 
                    }
                    children.clear();
                    break; 
                }
            }
        } else {
            // 如果没有输出重定向，并且是最后一个命令，则输出到当前 Shell 的 stdout
            command_builder.stdout(Stdio::inherit());
        }

        // 设置标准错误（通常不参与管道，直接重定向或继承）
        if let Some(filepath) = &parsed_cmd.stderr_redirect {
            match File::create(filepath) { 
                Ok(file) => {
                    command_builder.stderr(Stdio::from(file));
                },
                Err(e) => {
                    eprintln!("my_shell: 无法打开错误输出文件 {}: {}", filepath, e);
                    // 如果错误输出文件无法打开，则清除之前启动的子进程，并中断管道
                    for c in children.iter_mut() {
                        let _ = c.kill(); 
                    }
                    children.clear();
                    break; 
                }
            }
        } else {
            command_builder.stderr(Stdio::inherit());
        }

        // 尝试执行命令
        let child_spawn_result = command_builder.spawn();

        match child_spawn_result {
            Ok(mut child) => {
                if let Some(stdout) = child.stdout.take() {
                    previous_command_stdout = Some(stdout);
                }
                children.push(child);
            },
            Err(e) => {
                eprintln!("my_shell: {}: {}", parsed_cmd.name, e);
                // 如果命令执行失败，清除之前启动的子进程，并中断管道
                for c in children.iter_mut() {
                    let _ = c.kill(); // 尝试杀死子进程
                }
                children.clear();
                break; // 停止处理当前管道中的剩余命令
            },
        }
    }

    // 等待管道中的所有子进程完成
    for mut child in children.drain(..) {
        let _ = child.wait(); // 不关心输出，只等待完成
    }
} 