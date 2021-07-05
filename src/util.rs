use colored::*;
use structopt::StructOpt;
//------------------------------------------------
//------------APPLE STUFF-------------------------
//------------------------------------------------
pub fn apple_said(what: &str, more_info: &str) -> String {
    let s = match what {
        "yes" => format!("{}: {}", "Apple said yes", more_info).green().bold().to_string(),
        "no" =>  format!("{}: {}", "Apple said no", more_info).red().bold().to_string(),
        _ => String::from("")
    };
    s.to_string()
}

pub fn apple_said_no(more_info: &str) -> String {
    return apple_said("no", more_info);
}

pub fn apple_said_yes(more_info: &str) -> String {
    return apple_said("yes", more_info);
}

//------------------------------------------------
//------------CLI STUFF---------------------------
//------------------------------------------------

#[derive(Debug, StructOpt)]
pub struct Cli {
  #[structopt(long="nomute")]
  pub nomute: bool
}


