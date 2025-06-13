use std::env;

use rustyline::error::ReadlineError;
use rustyline::{Editor, Result as RlResult};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::completion::{Completer, Pair};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context};
use std::borrow::Cow;

mod parser;
mod executor;

// 定义一个辅助结构体，用于实现 rustyline 的 Completion、Hint 和 Highlight 特征
struct MyHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
    #[allow(dead_code)] // 允许死代码，因为这个字段是为了满足结构体要求，但实际值不被"读取"
    validator: (), // 不需要特殊的验证器
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> RlResult<(usize, Vec<Pair>)> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for MyHelper {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for MyHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{} [0m", hint))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for MyHelper {}
impl rustyline::Helper for MyHelper {}

// 用于文件路径补全的 completer (rustyline 内置)
use rustyline::completion::FilenameCompleter;
// 用于历史记录提示的 hinter (rustyline 内置)
use rustyline::hint::HistoryHinter;

fn main() {
    // 创建 rustyline 编辑器实例
    let config = rustyline::Config::builder()
        .history_ignore_space(true)
        .completion_type(rustyline::config::CompletionType::List)
        .edit_mode(rustyline::config::EditMode::Emacs)
        .build();

    let h = MyHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        validator: (),
    };
    let mut rl = Editor::with_config(config).expect("无法创建 Editor");
    rl.set_helper(Some(h));

    // 加载历史记录 (如果存在)
    // let history_path = "history.txt";
    // if rl.load_history(history_path).is_err() {
    //     println!("没有找到历史记录文件: {}. 创建新的历史记录.", history_path);
    // }

    loop {
        let readline = rl.readline("my_shell> "); // 使用 rustyline 读取输入

        match readline {
            Ok(command_line) => {
                let command_line = command_line.trim();
                if command_line.is_empty() {
                    continue;
                }

                // 将命令添加到历史记录
                rl.add_history_entry(command_line.to_string());

                // 解析用户输入的命令，可能包含管道
                let parsed_commands = match parser::parse_pipeline_commands(command_line) {
                    Ok(cmds) => cmds,
                    Err(e) => {
                        eprintln!("my_shell: 解析错误: {}", e);
                        continue;
                    }
                };

                // 处理内置命令 (只对管道中的第一个命令进行检查)
                // 确保 cd, exit, pwd 不会与其他外部命令通过管道组合
                if parsed_commands.len() == 1 {
                    let single_cmd = &parsed_commands[0];
                    match single_cmd.name.as_str() {
                        "exit" => {
                            println!("Exiting my_shell.");
                            break; // 退出主循环
                        },
                        "cd" => {
                            // 处理 cd 命令：改变当前工作目录
                            if single_cmd.args.len() == 0 {
                                eprintln!("cd: 缺少操作数");
                            } else if single_cmd.args.len() > 1 {
                                eprintln!("cd: 参数过多");
                            } else {
                                let path = &single_cmd.args[0];
                                if let Err(e) = env::set_current_dir(path) {
                                    eprintln!("cd: {}: {}", path, e);
                                }
                            }
                            continue; // cd 命令处理完毕，继续下一个循环
                        },
                        "pwd" => {
                            // 处理 pwd 命令：打印当前工作目录
                            match env::current_dir() {
                                Ok(path) => println!("{}", path.display()),
                                Err(e) => eprintln!("pwd: {}", e),
                            }
                            continue; // pwd 命令处理完毕，继续下一个循环
                        },
                        _ => { /* 不是内置命令，继续执行外部命令逻辑 */ }
                    }
                }

                // 执行管道中的命令
                executor::execute_pipeline(&parsed_commands);
            },
            Err(ReadlineError::Interrupted) => { // Ctrl-C
                println!("Ctrl-C 捕获，退出.");
                break;
            },
            Err(ReadlineError::Eof) => { // Ctrl-D
                println!("Ctrl-D 捕获，退出.");
                break;
            },
            Err(err) => {
                eprintln!("读取命令行错误: {:?}", err);
                break;
            }
        }
    }

    // 保存历史记录
    // if let Err(err) = rl.save_history(history_path) {
    //     eprintln!("保存历史记录错误: {:?}", err);
    // }
}
