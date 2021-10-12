mod error;
use error::Error;
mod operation;
use operation::Operation;

pub fn run(operation: Operation) -> Result<(), Error> {
    match operation {
	Operation::Index(path) => {},
	Operation::Scan(path) => {},
    }
    todo!();
}
