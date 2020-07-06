use failure::Error;
use structopt::StructOpt;

mod doc;

#[derive(StructOpt, Debug)]
enum Opt {
    Doc(doc::DocOpt),
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    match &opt {
        Opt::Doc(doc_opt) => doc::command(doc_opt),
    }
}
