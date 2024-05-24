use std::{
    collections::HashMap,
    env,
    fmt::Display,
    io::Write,
    ops::{AddAssign, SubAssign},
    str::FromStr,
};

fn main() {
    println!("Hello, world!");
    let program = parse();
    println!("{:#?}", program);
    let linked: PostLinkerProgram<usize, f64> = link(program);
    println!("{:#?}", linked);
    let interpreter = LmcInterpreter::new(linked);
    for step in interpreter {
        println!("{:?}", step);
    }
}
struct LmcInterpreter<Addr: Clone, Data: Clone> {
    program: PostLinkerProgram<Addr, Data>,
    program_counter: Addr,
    accumulator: Data,
}
impl<Addr: Clone + Default, Data: Clone + Default> LmcInterpreter<Addr, Data> {
    fn new(program: PostLinkerProgram<Addr, Data>) -> Self {
        LmcInterpreter {
            program,
            program_counter: Addr::default(),
            accumulator: Data::default(),
        }
    }
}
#[derive(Debug)]
struct LmcStatus<Addr: Clone, Data: Clone> {
    pc: Addr,
    accumulator: Data,
    instruction: LmcInstruction<Addr, Data>,
}
impl<
        Data: Clone
            + FromStr<Err: std::fmt::Debug>
            + PartialEq
            + PartialOrd
            + Display
            + AddAssign
            + SubAssign
            + From<u8>,
    > Iterator for LmcInterpreter<usize, Data>
where
    usize:
        std::slice::SliceIndex<[LmcInstruction<usize, Data>], Output = LmcInstruction<usize, Data>>,
{
    type Item = LmcStatus<usize, Data>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut state = LmcStatus {
            pc: self.program_counter,
            accumulator: self.accumulator.clone(),
            instruction: self.program.0[self.program_counter].clone(),
        };
        match self.program.0[self.program_counter].clone() {
            LmcInstruction::Inp => {
                print!("Inp: ");
                let _ = std::io::stdout().flush();
                let mut input = String::default();
                let _ = std::io::stdin().read_line(&mut input);
                self.accumulator = input.trim().parse().unwrap();
                self.program_counter += 1;
            }
            LmcInstruction::Add(index) => {
                self.accumulator += self.program.0[index].value().unwrap().clone();
                self.program_counter += 1;
            }
            LmcInstruction::Sub(index) => {
                self.accumulator -= self.program.0[index].value().unwrap().clone();
                self.program_counter += 1;
            }
            LmcInstruction::Sta(index) => {
                *self.program.0[index].value_mut().unwrap() = self.accumulator.clone();
                self.program_counter += 1;
            }
            LmcInstruction::Lda(index) => {
                self.accumulator = self.program.0[index].value().unwrap().to_owned();
                self.program_counter += 1;
            }
            LmcInstruction::Bra(index) => {
                self.program_counter = index;
            }
            LmcInstruction::Brz(index) => {
                if self.accumulator == Data::from(0) {
                    self.program_counter = index;
                } else {
                    self.program_counter += 1;
                }
            }
            LmcInstruction::Brp(index) => {
                if self.accumulator >= Data::from(0) {
                    self.program_counter = index;
                } else {
                    self.program_counter += 1;
                }
            }
            LmcInstruction::Out => {
                println!("OUT: {}", self.accumulator);
                self.program_counter += 1;
            }
            LmcInstruction::Hlt => return None,
            LmcInstruction::Dat(value) => panic!("Attempted to execute \"DAT\" memory address"),
        }
        state.accumulator = self.accumulator.clone();
        Some(state)
    }
}
#[derive(Debug, Clone)]
enum LmcInstruction<AddressType: Clone, DataType: Clone> {
    Inp,
    Add(AddressType),
    Sub(AddressType),
    Sta(AddressType),
    Lda(AddressType),
    Bra(AddressType),
    Brz(AddressType),
    Brp(AddressType),
    Out,
    Hlt,
    Dat(DataType),
}
impl<Addr: Clone, Data: Clone> LmcInstruction<Addr, Data> {
    fn value(&self) -> Option<&Data> {
        if let LmcInstruction::Dat(value) = self {
            return Some(value);
        } else {
            return None;
        }
    }
    fn value_mut(&mut self) -> Option<&mut Data> {
        if let LmcInstruction::Dat(value) = self {
            return Some(value);
        } else {
            return None;
        }
    }
    fn is_data(&self) -> bool {
        if let LmcInstruction::Dat(_) = self {
            true
        } else {
            false
        }
    }
    fn operand<'a>(&'a self) -> Option<&'a Addr> {
        match self {
            LmcInstruction::Add(operand) => Some(operand),
            LmcInstruction::Sub(operand) => Some(operand),
            LmcInstruction::Sta(operand) => Some(operand),
            LmcInstruction::Lda(operand) => Some(operand),
            LmcInstruction::Bra(operand) => Some(operand),
            LmcInstruction::Brz(operand) => Some(operand),
            LmcInstruction::Brp(operand) => Some(operand),
            LmcInstruction::Dat(_) => None,
            LmcInstruction::Out | LmcInstruction::Hlt | LmcInstruction::Inp => None,
        }
    }
}
impl<U: FromStr + Clone> TryFrom<(&str, Option<&str>)> for LmcInstruction<String, U> {
    type Error = TryIntoLmcInstructionError;
    fn try_from(value: (&str, Option<&str>)) -> Result<Self, Self::Error> {
        Ok(match value.0.to_lowercase().trim() {
            "add" => LmcInstruction::Add(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .to_string(),
            ),
            "sub" => LmcInstruction::Sub(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .to_string(),
            ),
            "sta" => LmcInstruction::Sta(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .to_string(),
            ),
            "lda" => LmcInstruction::Lda(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .to_string(),
            ),
            "bra" => LmcInstruction::Bra(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .to_string(),
            ),
            "brz" => LmcInstruction::Brz(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .to_string(),
            ),
            "brp" => LmcInstruction::Brp(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .to_string(),
            ),
            "dat" => LmcInstruction::Dat(
                value
                    .1
                    .ok_or(TryIntoLmcInstructionError::MissingOperand)?
                    .trim()
                    .parse()
                    .map_err(|_| {
                        TryIntoLmcInstructionError::InvalidOperand(
                            value.1.unwrap().trim().to_string(),
                        )
                    })?,
            ),
            "inp" => LmcInstruction::Inp,
            "out" => LmcInstruction::Out,
            "hlt" => LmcInstruction::Hlt,
            op => Err(TryIntoLmcInstructionError::UnknownOperator(op.to_string()))?,
        })
    }
}
#[derive(Clone, Debug)]
enum TryIntoLmcInstructionError {
    UnknownOperator(String),
    MissingOperand,
    InvalidOperand(String),
}
#[derive(Debug, Clone)]
struct PreLinkerProgram<T: Clone>(Vec<LmcInstruction<String, T>>, HashMap<String, usize>);
impl<T: Clone> Default for PreLinkerProgram<T> {
    fn default() -> Self {
        Self(Vec::new(), HashMap::new())
    }
}
#[derive(Debug, Clone)]
struct PostLinkerProgram<Addr: Clone, Data: Clone>(Vec<LmcInstruction<Addr, Data>>);
fn link<Addr: From<usize> + Clone, Data: FromStr + Clone>(
    initial: PreLinkerProgram<String>,
) -> PostLinkerProgram<Addr, Data>
where
    <Data as FromStr>::Err: std::fmt::Debug,
{
    let mut linked: PostLinkerProgram<Addr, Data> = PostLinkerProgram(Vec::new());
    for instruction in initial.0.iter() {
        linked.0.push(match instruction {
            LmcInstruction::Add(operand) => LmcInstruction::Add(initial.1[operand].into()),
            LmcInstruction::Sub(operand) => LmcInstruction::Sub(initial.1[operand].into()),
            LmcInstruction::Sta(operand) => LmcInstruction::Sta(initial.1[operand].into()),
            LmcInstruction::Lda(operand) => LmcInstruction::Lda(initial.1[operand].into()),
            LmcInstruction::Bra(operand) => LmcInstruction::Bra(initial.1[operand].into()),
            LmcInstruction::Brz(operand) => LmcInstruction::Brz(initial.1[operand].into()),
            LmcInstruction::Brp(operand) => LmcInstruction::Brp(initial.1[operand].into()),
            LmcInstruction::Dat(operand) => LmcInstruction::Dat(operand.parse().unwrap()),
            LmcInstruction::Out => LmcInstruction::Out,
            LmcInstruction::Hlt => LmcInstruction::Hlt,
            LmcInstruction::Inp => LmcInstruction::Inp,
        });
    }
    linked
}
fn parse<Data: FromStr<Err: std::fmt::Debug> + Clone>() -> PreLinkerProgram<Data> {
    let mut program = PreLinkerProgram::default();
    let mut immediates = Vec::new();
    for (index, line) in
        std::fs::read_to_string(env::args().skip(1).next().expect("No file provided"))
            .unwrap()
            .lines()
            .map(|line| format_code(line))
            .flatten()
            .skip_while(|line| line.trim().is_empty())
            .enumerate()
    {
        let Some((label, instr_and_operand)) = line.split_once(" ") else {
            continue;
        };
        let (instruction, operand) =
            if let Some((instruction, operand)) = instr_and_operand.split_once(" ") {
                (instruction.trim(), Some(operand.trim()))
            } else {
                (instr_and_operand.trim(), None)
            };
        if let Some(op) = operand {
            if op.starts_with("#") {
                immediates.push(op.to_owned());
            }
        }
        let code: LmcInstruction<String, Data> = (instruction, operand).try_into().unwrap();
        program.0.push(code);
        if !label.trim().is_empty() {
            program.1.insert(label.trim().to_string(), index);
        }
    }
    for immediate in immediates {
        match program.1.entry(immediate.to_string()) {
            std::collections::hash_map::Entry::Occupied(_) => continue,
            std::collections::hash_map::Entry::Vacant(entry) => entry.insert(program.0.len()),
        };
        program
            .0
            .push(LmcInstruction::Dat(immediate[1..].parse().unwrap()));
    }
    program
}
fn format_code(line: &str) -> Option<String> {
    let mut was_whitespace = false;
    let mut out = Vec::new();
    let line = if let Some((code, _)) = line.split_once("//") {
        code
    } else {
        line
    };
    if line.trim().is_empty() {
        return None;
    }
    for index in 0_usize.. {
        let Some(byte) = line.as_bytes().get(index) else {
            break;
        };
        let chr = char::from(*byte);
        if chr.is_whitespace() {
            if !was_whitespace {
                out.push(' ');
                was_whitespace = true;
            }
        } else {
            out.push(chr);
            was_whitespace = false;
        }
    }
    Some(out.iter().collect())
}
