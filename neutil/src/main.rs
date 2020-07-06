use failure::Error;
use structopt::StructOpt;

mod doc;
mod info;

#[derive(StructOpt, Debug)]
enum Opt {
    Doc(doc::DocOpt),
    Info(info::InfoOpt),
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    match &opt {
        Opt::Doc(doc_opt) => doc::command(doc_opt),
        Opt::Info(info_opt) => info::command(info_opt),
    }
}
