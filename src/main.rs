use dirs::home_dir;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{env, fmt, fs::File, io::Write};
use text_colorizer::*;

#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    title: String,
    due: Option<String>,
    done: bool,
}

enum TodoPicker {
    Index(usize),
    Title(String),
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            if self.done { "[x]" } else { "[ ]" },
            self.title,
            if self.due.is_some() {
                "(".to_owned() + &self.due.clone().unwrap() + ")"
            } else {
                String::from("")
            },
        )
    }
}

const USAGE: &str = r"
USAGE
todo [commmand] [arguments...]

COMMANDS
todo                                prints todo list
todo add <title>                    add new item to todo list
todo edit <item_id> <new_title>     edits an item title
todo done <title_or_item_id>        marks an item as done
todo undone <title_or_item_id>      marks an item as undone
todo delete <title_or_item_id>      deletes an item
todo delete done                    deletes all items marked done
";

fn parse_arg(todos: &mut Vec<Todo>) {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 0 {
        print_todo(todos, None);
    } else if args.len() == 1 {
        println!(
            "{} {}",
            args[0].to_string().red().bold(),
            "what?!".to_string().red().bold(),
        );
        println!("{}", USAGE);
    } else {
        match args[0].as_str() {
            "add" => add(todos, args[1..].join(" "), None),
            "edit" => edit(
                todos,
                args[1].trim().parse::<usize>().expect(USAGE),
                args[2..].join(" "),
                None,
            ),
            "done" => done(todos, parse_picker(args[1].clone())),
            "undone" => undone(todos, parse_picker(args[1].clone())),
            "delete" => {
                if args[1] == "done" {
                    delete_done(todos);
                } else {
                    delete(todos, parse_picker(args[1].clone()));
                }
            }
            _ => println!("{}", USAGE),
        }
    };
}

fn parse_picker(arg: String) -> TodoPicker {
    match arg.trim().parse::<usize>() {
        Ok(num) => TodoPicker::Index(num),
        Err(_) => TodoPicker::Title(arg.clone()),
    }
}

fn read_data() -> Vec<Todo> {
    let file_path = home_dir().expect("Can't find home dir!").join(".todo");
    if !file_path.exists() {
        let mut file = File::create(&file_path).expect("Can't create file!");
        file.write_all(b"[]").expect("Can't write to file!")
    }
    let json = std::fs::read_to_string(file_path).unwrap();
    serde_json::from_str(&json).expect("Corrupted data")
}

fn write_data(todos: &Vec<Todo>) {
    let file_path = home_dir().expect("Can't find home dir!").join(".todo");
    let json = serde_json::to_string(&todos).unwrap();
    std::fs::write(file_path, json).unwrap();
}

fn print_todo(todos: &Vec<Todo>, highlight: Option<usize>) {
    let is_h = highlight.is_some();
    for (i, todo) in todos.iter().enumerate() {
        if is_h && i == highlight.unwrap() {
            println!("{}. {}", i, todo.to_string().blue().bold());
        } else {
            println!("{}. {}", i, todo);
        }
    }
    if todos.len() == 0 {
        println!("{}", "Nothing to do".green().bold());
    }
}

fn add(todos: &mut Vec<Todo>, title: String, due: Option<String>) {
    let todo = Todo {
        title: title,
        due: due,
        done: false,
    };
    let _ = &todos.push(todo);
    print_todo(todos, Some(todos.len() - 1))
}

fn edit(todos: &mut Vec<Todo>, index: usize, title: String, due: Option<String>) {
    if index < todos.len() {
        if !title.is_empty() {
            todos[index].title = title;
        }
        todos[index].due = due;
        print_todo(todos, Some(index));
    } else {
        println!("{}", "Edit what?!".red().bold());
    }
}

fn done(todos: &mut Vec<Todo>, picker: TodoPicker) {
    done_toggler(todos, picker, true);
}

fn undone(todos: &mut Vec<Todo>, picker: TodoPicker) {
    done_toggler(todos, picker, false);
}

fn done_toggler(todos: &mut Vec<Todo>, picker: TodoPicker, done: bool) {
    let mut changed: Option<usize> = None;
    match picker {
        TodoPicker::Index(index) => {
            if index < todos.len() {
                changed = Some(index);
            }
        }
        TodoPicker::Title(title) => {
            for (i, todo) in todos.iter_mut().enumerate() {
                if todo.title.to_lowercase().starts_with(&title.to_lowercase()) {
                    changed = Some(i);
                }
            }
        }
    }
    if let Some(i) = changed {
        todos[i].done = done;
        print_todo(&todos, changed);
    } else {
        println!("{}", "Done what?!".red().bold());
    }
}

fn delete(todos: &mut Vec<Todo>, picker: TodoPicker) {
    let mut delete_id: Option<usize> = None;
    match picker {
        TodoPicker::Index(index) => {
            if index < todos.len() {
                delete_id = Some(index);
            }
        }
        TodoPicker::Title(title) => {
            for (i, todo) in todos.iter_mut().enumerate() {
                if todo.title.to_lowercase().starts_with(&title.to_lowercase()) {
                    delete_id = Some(i);
                    break;
                }
            }
        }
    }
    if let Some(id) = delete_id {
        let removed = todos.remove(id);
        println!("\x1B[9m{}\x1B[0m", removed.to_string().red());
        print_todo(&todos, None);
    } else {
        println!("{}", "Delete what?!".red().bold());
    }
}

fn delete_done(todos: &mut Vec<Todo>) {
    let mut removed: Vec<Todo> = vec![];
    let mut i = 0;
    while i < todos.len() {
        if todos[i].done == false {
            i += 1
        } else {
            let r = todos.remove(i);
            removed.push(r);
        }
    }
    for i in removed.iter() {
        println!("\x1B[9m{}\x1B[0m", i.to_string().red());
    }
    print_todo(todos, None);
}

fn main() {
    let mut todos = read_data();
    parse_arg(&mut todos);
    write_data(&todos);
}
