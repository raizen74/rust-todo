mod api;
use to_do_dal::to_do_items::enums::TaskStatus;
use clap::Parser;
use glue::errors::NanoServiceError;
use to_do_dal::to_do_items::schema::NewToDoItem;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    title: String,
    #[arg(short, long)]
    status: String,
}
fn main() -> Result<(), NanoServiceError> {
    let args = Args::parse();
    let status_enum = TaskStatus::from_string(&args.status)?;
    let item = NewToDoItem {
            title: args.title,
            status: status_enum,
        };
    println!("{}", item);
    Ok(())
}
