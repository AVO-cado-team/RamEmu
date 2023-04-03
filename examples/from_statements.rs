use ramemu::program::Program;
use ramemu::ram::Ram;
use ramemu::stmt::{Stmt, Value};
use std::io::BufReader;
use std::io::BufWriter;

fn main() {
  let program = Program::from(vec![
    Stmt::Load(Value::Pure(2), 1),
    Stmt::Add(Value::Pure(2), 3),
    Stmt::Output(Value::Pure(0), 4),
    Stmt::Halt(5),
  ]);

  let reader = BufReader::new(std::io::empty());
  let writer = BufWriter::new(std::io::sink());
  let mut ram = Ram::new(program, Box::new(reader), Box::new(writer));

  ram.run().unwrap();
  assert_eq!(ram.get_registers().get(0), 4);
}
