use std::process::{Command, Output};

use errors::{Result, ResultExt};

pub trait HandleScript {
    fn handle(&self, script: &str) -> Result<()>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ScriptHandler {
    PrintScript,
    RunScript,
}


impl HandleScript for ScriptHandler {
    fn handle(&self, script: &str) -> Result<()> {
        match *self {
            ScriptHandler::PrintScript => {
                println!("{}", script);
                Ok(())
            }
            ScriptHandler::RunScript => run_script(script),
        }
    }
}

fn fail_script(script: &str, output: Option<Output>) -> String {
    format!("Failed to run script, {}:\noutput:\n{:#?}", script, output)
}

fn run_script(script: &str) -> Result<()> {
    Command::new("sh")
        .arg("-c")
        .arg(script)
        .output()
        .chain_err(|| fail_script(script, None))
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(fail_script(script, Some(output)).into())
            }
        })
}
