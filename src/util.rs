use colored::*;
use structopt::StructOpt;
//------------------------------------------------
//------------APPLE STUFF-------------------------
//------------------------------------------------
enum What {
    Yes,
    No
}
fn apple_said(what: What, more_info: &str) -> String {
    let s = match what {
        What::Yes => format!("{}: {}", "Apple said yes", more_info).green().bold().to_string(),
        What::No =>  format!("{}: {}", "Apple said no", more_info).red().bold().to_string(),
    };
    s.to_string()
}

pub fn apple_said_no(more_info: &str) -> String {
    return apple_said(What::Yes, more_info);
}

pub fn apple_said_yes(more_info: &str) -> String {
    return apple_said(What::No, more_info);
}

//------------------------------------------------
//------------CLI STUFF---------------------------
//------------------------------------------------

#[derive(Debug, StructOpt)]
pub struct Cli {
  #[structopt(long="nomute")]
  pub nomute: bool
}


