# my-shell 项目开发文档
> 在我windows电脑的wsl里写，重新配git有点麻烦
> 软2312 秦一鸣 20232241198
## 项目简介

`my-shell` 是一个用 Rust 编写的简易命令行 Shell 程序。它能够读取用户输入的命令，支持管道（`|`）操作，并实现了 `cd`（切换目录）和 `exit`（退出）等内建命令。该项目主要用于学习和演示如何用 Rust 实现一个基础的 Shell。

---

## 目录结构

```
my_shell/
├── src/
│   └── main.rs   # 主程序文件，包含所有核心逻辑
├── Cargo.toml    # Rust 项目配置文件
```

---

## 主要功能

1. **命令行交互**：循环读取用户输入，解析并执行命令。
2. **内建命令**：支持 `cd`（切换目录）和 `exit`（退出 Shell）。
3. **管道支持**：支持多个命令通过 `|` 管道连接，前一个命令的输出作为下一个命令的输入。
4. **错误处理**：对输入、命令执行、管道等环节进行错误提示和资源清理。

---

## 代码结构与实现思路

### 1. 主循环 (`main` 函数)

- 作用：不断读取用户输入，处理命令，直到用户输入 `exit` 或 EOF（Ctrl+D）。
- 主要流程：
  1. 打印提示符 `my_shell> `。
  2. 读取一行用户输入。
  3. 判断输入内容：
     - 为空则继续下一轮循环。
     - 为 `exit` 则退出 Shell。
     - 其他情况调用 `execute_pipeline` 处理命令（支持管道）。

### 2. 命令执行与管道处理 (`execute_pipeline` 函数)

- 作用：解析并执行用户输入的命令，支持管道。
- 主要流程：
  1. 按 `|` 分割命令，得到每个子命令。
  2. 遍历每个子命令，依次处理：
     - 检查命令是否为空，若为空则报错并清理已启动的子进程。
     - 解析命令名和参数。
     - 特殊处理 `cd` 命令（只能单独使用，不能在管道中）。
     - 其他命令则通过 `Command` 启动子进程。
     - 管道实现：前一个命令的 `stdout` 作为下一个命令的 `stdin`。
     - 最后一个命令的输出直接继承终端。
  3. 所有子进程启动后，等待它们执行完毕，并检查返回状态。

### 3. 内建命令处理

- `cd`：切换当前工作目录，直接影响父进程（Shell 本身），不能在管道中使用。
- `exit`：退出 Shell，结束主循环。

### 4. 错误处理与资源清理

- 输入错误、命令执行失败、管道错误等都会输出详细错误信息。
- 如果管道中某个命令失败，会尝试终止并回收已启动的子进程，避免僵尸进程。

---

## 关键代码片段解析

### 主循环

```rust
loop {
    print!("my_shell> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => { println!("\nExiting my_shell."); break; }
        Ok(_) => {
            let trimmed_input = input.trim();
            if trimmed_input.is_empty() { continue; }
            if trimmed_input == "exit" { println!("Exiting my_shell."); break; }
            execute_pipeline(trimmed_input);
        }
        Err(error) => { eprintln!("Error reading input: {}", error); }
    }
}
```

### 管道与命令执行

```rust
fn execute_pipeline(line: &str) {
    let commands_str: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
    let num_commands = commands_str.len();
    let mut children: Vec<Child> = Vec::new();
    let mut previous_stdout: Option<Stdio> = None;

    for (i, command_segment) in commands_str.iter().enumerate() {
        // ... 解析命令名和参数 ...
        // ... 处理 cd 命令 ...
        let mut current_command = Command::new(command_name);
        current_command.args(&args);

        if let Some(prev_stdout_handle) = previous_stdout.take() {
            current_command.stdin(prev_stdout_handle);
        }
        if i < num_commands - 1 {
            current_command.stdout(Stdio::piped());
        } else {
            current_command.stdout(Stdio::inherit());
        }
        // ... 启动子进程，错误处理 ...
    }
    // ... 等待所有子进程结束 ...
}
```

### `cd` 命令特殊处理

```rust
if command_name == "cd" {
    if num_commands > 1 {
        eprintln!("'cd' cannot be part of a pipeline.");
        // ... 清理子进程 ...
        return;
    }
    let new_dir = args.get(0).map_or_else(
        || env::var("HOME").unwrap_or_else(|_| "/".to_string()),
        |x| x.to_string()
    );
    let root = Path::new(&new_dir);
    if let Err(e) = env::set_current_dir(&root) {
        eprintln!("Error changing directory to {}: {}", new_dir, e);
    }
    return;
}
```

---

## 运行与测试

1. **编译项目**  
   在项目根目录下运行：
   ```bash
   cargo build
   ```

2. **运行 Shell**
   ```bash
   cargo run
   ```

3. **示例操作**
   ```
   my_shell> ls -l
   my_shell> cat file.txt | grep hello | wc -l
   my_shell> cd /tmp
   my_shell> exit
   ```

---

## 总结

- 本项目通过 Rust 的 `std::process::Command` 实现了外部命令的启动与管道连接。
- 通过对 `cd`、`exit` 等内建命令的特殊处理，实现了 Shell 的基本功能。
- 错误处理和资源清理较为完善，适合学习和扩展。

---

如需进一步扩展功能（如重定向、环境变量、命令历史等），可在此基础上继续开发。
