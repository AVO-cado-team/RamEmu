use std::error::Error;

use crate::stmt::Label;
use crate::stmt::RegisterValue;
use crate::stmt::Stmt;
use crate::stmt::Value;

pub fn parse(source: &str) -> impl Iterator<Item = Result<Stmt, ParseError>> + '_ {
  source
    .lines()
    .enumerate()
    .map(|(i, l)| (i, l.trim()))
    .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
    .map(|(i, l)| parse_line(l, i))
}


pub fn parse_line(line: &str, index: usize) -> Result<Stmt, ParseError> {
  let facts: Vec<_> = line.split_whitespace().collect();

  if facts.len() > 2 && !facts[2].starts_with('#') {
    Err(ParseError::UnsupportedSyntax(index))?
  }

  let first = facts.first().ok_or(ParseError::UnknownError(index))?;
  let tail = facts.get(1);

  if let Some(label) = first.strip_suffix(':') {
    if is_valid_label(label) {
      return Ok(Stmt::Label(label.to_string(), index));
    }
    Err(ParseError::LabelIsNotValid(index))?
  }

  let opcode = first.to_uppercase();

  let stmt = match opcode.as_str() {
    "LOAD" | "ADD" | "SUB" | "MUL" | "DIV" | "WRITE" | "OUTPUT" => {
      parse_with_value(&opcode, tail.ok_or(ParseError::ArgumentIsRequired(index))?, index)?
    }
    "JUMP" | "JMP" | "JZ" | "JZERO" | "JGZ" | "JGTZ" => {
      parse_with_label(&opcode, tail.ok_or(ParseError::ArgumentIsRequired(index))?, index)?
    }
    "STORE" | "INPUT" | "READ" => parse_with_register(&opcode, tail.ok_or(ParseError::ArgumentIsRequired(index))?, index)?,
    "HALT" => Stmt::Halt(index),
    _ => Err(ParseError::UnsupportedOpcode(index, opcode))?,
  };

  Ok(stmt)
}

fn parse_with_register(opcode: &str, tail: &str, index: usize) -> Result<Stmt, ParseError> {
  let arg: RegisterValue = {
    if let Some(tail) = tail.strip_prefix('*') {
      RegisterValue::Indirect(tail.parse().map_err(|_| ParseError::argument_value_must_be_numeric(index))?)
    } else if let Ok(arg) = tail.parse::<usize>() {
      RegisterValue::Direct(arg)
    } else if tail.starts_with('=') {
      Err(ParseError::pure_argument_not_allowed(index))?
    } else {
      Err(ParseError::not_valid_argument(index))?
    }
  };
  match opcode {
    "STORE" => Ok(Stmt::Store(arg, index)),
    "INPUT" | "READ" => Ok(Stmt::Input(arg, index)),
    _ => !unreachable!("Opcodes were chenged in parse function, but not there"), 
  }
}

fn parse_with_value(head: &str, tail: &str, index: usize) -> Result<Stmt, ParseError> {
  let arg: Value = {
    if let Some(tail) = tail.strip_prefix('=') {
      Value::Pure(tail.parse().map_err(|_| ParseError::argument_value_must_be_numeric(index))?)
    } else if let Some(tail) = tail.strip_prefix('*') {
      Value::Register(RegisterValue::Indirect(
        tail.parse().map_err(|_| ParseError::argument_value_must_be_numeric(index))?,
      ))
    } else if let Ok(arg) = tail.parse::<usize>() {
      Value::Register(RegisterValue::Direct(arg))
    } else {
      Err(ParseError::not_valid_argument(index))?
    }
  };

  match head {
    "LOAD" => Ok(Stmt::Load(arg, index)),
    "OUTPUT" | "WRITE" => Ok(Stmt::Output(arg, index)),
    "ADD" => Ok(Stmt::Add(arg, index)),
    "SUB" => Ok(Stmt::Sub(arg, index)),
    "MUL" => Ok(Stmt::Mul(arg, index)),
    "DIV" => Ok(Stmt::Div(arg, index)),
    _ => !unreachable!("Opcodes were chenged in parse function, but not there"),
  }
}

fn parse_with_label(head: &str, tail: &str, index: usize) -> Result<Stmt, ParseError> {
  let label: Label = if is_valid_label(tail) {
    Label::new(tail.to_string())
  } else {
    Err(ParseError::LabelIsNotValid(index))?
  };

  match head {
    "JUMP" | "JMP" => Ok(Stmt::Jump(label, index)),
    "JZ" | "JZERO" => Ok(Stmt::JumpIfZero(label, index)),
    "JGZ" | "JGTZ" => Ok(Stmt::JumpGreatherZero(label, index)),
    _ => !unreachable!("Opcodes were chenged in parse function, but not there"),
  }
}

fn is_valid_label(label: &str) -> bool {
  let Some(first) = label.chars().next() else { return false };

  if !first.is_ascii_alphabetic() && first != '_' {
    return false;
  }

  label
    .chars()
    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c.is_ascii_digit())
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum ParseError {
  LabelIsNotValid(usize),

  UnsupportedSyntax(usize),
  UnsupportedOpcode(usize, String),

  ArgumentIsRequired(usize),
  ArgumentIsNotValid(usize, InvalidArgument),
  
  UnknownError(usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum InvalidArgument {
  LabelIsNotValid,
  ArgumentIsRequired,
  ArgumentValueMustBeNumberic,
  PureArgumentIsNotAllowed,

  ArgumentIsNotValid,
}


impl ParseError {
  fn pure_argument_not_allowed(index: usize) -> Self {
    ParseError::ArgumentIsNotValid(
      index, 
      InvalidArgument::PureArgumentIsNotAllowed
    )
  }

  fn not_valid_argument(index: usize) -> Self {
    ParseError::ArgumentIsNotValid(
      index,
      InvalidArgument::ArgumentIsNotValid
    )
  }

  fn argument_value_must_be_numeric(index: usize) -> Self {
    ParseError::ArgumentIsNotValid(
      index,
      InvalidArgument::ArgumentValueMustBeNumberic
    )
  }
}

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Parse error")
  }
}

impl Error for ParseError {}



#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_label_is_not_valid() {
    let line = "фывфыфыв:";
    let result = parse_line(line, 0);
    
    assert_eq!(
      result, 
      Err(ParseError::LabelIsNotValid(0))
    );
  }

  #[test]
  fn test_unsupported_syntax() {
    let line = "LOAD 1 2";
    let result = parse_line(line, 0);
    
    assert_eq!(
      result, 
      Err(ParseError::UnsupportedSyntax(0))
    );
  }

  #[test]
  fn test_unsupported_opcode() {
    let line = "KoKotinf 1 2";
    let result = parse_line(line, 0);
    
    assert_eq!(
      result, 
      Err(ParseError::UnsupportedSyntax(0))
    );
  }

  #[test]
  fn test_argument_is_required() {
    let line = "LOAD";
    let result = parse_line(line, 0);
    
    assert_eq!(
      result, 
      Err(ParseError::ArgumentIsRequired(0))
    );
  }

  #[test]
  fn test_pure_argument_not_allowed() {
    let line = "STORE =1";
    let result = parse_line(line, 0);
    
    assert_eq!(
      result, 
      Err(ParseError::pure_argument_not_allowed(0))
    );
  }

  #[test]
  fn test_argument_value_must_be_numeric() {
    let line = "STORE *a";
    let result = parse_line(line, 0);
    
    assert_eq!(
      result, 
      Err(ParseError::argument_value_must_be_numeric(0))
    );
  }

  #[test]
  fn test_argument_is_not_valid() {
    let line = "STORE a";
    let result = parse_line(line, 0);
    
    assert_eq!(
      result, 
      Err(ParseError::not_valid_argument(0))
    );
  }
  
}