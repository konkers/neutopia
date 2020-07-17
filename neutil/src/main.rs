use failure::Error;
use structopt::StructOpt;

mod checks;
mod doc;
mod info;
mod password;

#[derive(StructOpt, Debug)]
enum Opt {
    Checks(checks::ChecksOpt),
    Doc(doc::DocOpt),
    Info(info::InfoOpt),
    Password(password::PasswordOpt),
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    match &opt {
        Opt::Checks(checks_opt) => checks::command(checks_opt),
        Opt::Doc(doc_opt) => doc::command(doc_opt),
        Opt::Info(info_opt) => info::command(info_opt),
        Opt::Password(password_opt) => password::command(password_opt),
    }
}
