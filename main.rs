use nix::unistd::{pipe, fork, ForkResult, close, dup2, execvp};
use std::ffi::CString;
use std::env;
use std::io::{self, Write};
use nix::sys::wait::waitpid;


fn main() -> io::Result<()> {
    loop {
        let directory = env::current_dir();

        struct Command{
            input_file: String,
            output_file: String,
            pipe: Vec<String>,
        }

        println!("{:#?}",directory.unwrap());
        let input = get_input("What command would you like to run?")?;

        let pip: Vec<String> = input.split("|").map(|s| s.to_string()).collect();

        let mut commands = Command{
            input_file: " ".to_string(),
            output_file: " ".to_string(),
            pipe: pip.clone(),
        };

        if pip[0].contains("<"){
            let parts: Vec<&str> = pip[0].split("<").collect();
            println!("{:?}",parts);
            commands.input_file = parts[1].to_string();

        }
        if pip[pip.len()-1].contains(">"){
            let parts2: Vec<&str >= pip[0].split("<").collect();
            println!("{:?}",parts2);
            commands.output_file = parts2[1].to_string();
        }

        println!("{}",commands.pipe[0].to_string());

        if input.len() > 0 {
            let external = externalize(&input);
            if input == "exit"{
                println!("Closing process..... Done");
                break;
            }
            if external[0].to_str() == Ok("cd"){
                let directory = external[1].to_str();
                env::set_current_dir(directory.unwrap());
            }
            else {
                match unsafe {fork()}.unwrap() {
                    ForkResult::Parent{child} => {
                        println!("wc pid is {}", child);
                        waitpid(child, Option::None).unwrap();
                        println!("Finished! Exiting...");
                    },

                    ForkResult::Child => {
                        let (wc_in, grep_out) = pipe().unwrap();

                        match unsafe {fork()}.unwrap() {
                            ForkResult::Parent{child} => {
                                close(grep_out).unwrap();

                                println!("grep pid is {}", child);
                                dup2(wc_in, 0).unwrap();
                                let array = externalize(pip[pip.len() - 1].as_str());
                                execvp(&array[0], &*array).unwrap();
                            },

                            ForkResult::Child => {
                                close(wc_in).unwrap();

                                let (grep_in, ls_out) = pipe().unwrap();

                                match unsafe {fork()}.unwrap() {
                                    ForkResult::Parent {child} => {
                                        close(ls_out).unwrap();

                                        println!("ls pid is {}", child);
                                        dup2(grep_out, 1).unwrap();
                                        dup2(grep_in, 0).unwrap();
                                        let array = externalize(pip[pip.len() - 1].as_str());
                                        execvp(&array[0], &*array).unwrap();
                                    },

                                    ForkResult::Child => {
                                        close(grep_in).unwrap();

                                        dup2(ls_out, 1).unwrap();
                                        let array = externalize(pip[0].as_str());
                                        execvp(&array[0], &*array).unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            }
        }
    Ok(())
    }



fn externalize(command: &str) -> Box<[CString]>{
    let converted = command.split_whitespace()
        .map(|s| CString::new(s).unwrap())
        .collect::<Vec<_>>();
    converted.into_boxed_slice()
}

// Function used you interpret user input in the terminal and get it into a for the program can
// read
fn get_input(prompt: &str) -> io::Result<String> {
    let mut buffer = String::new();
    print!("{} ", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut buffer)?;
    buffer.pop();
    Ok(buffer)
}
