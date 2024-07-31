use anyhow::Context;
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use gimli::*;
use object::{Object, read::ObjectSection};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecodedInstruction {
    pub address: usize,
    pub name: String,
    pub operands: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObjectFile {
    pub debug_info: BTreeMap<usize, Vec<SourceLocation>>, // Address to code region
    pub text_section: Vec<DecodedInstruction>,
}

pub fn load_object(path: impl AsRef<Path>) -> anyhow::Result<ObjectFile> {
    let data = fs::read(path)?;
    let file = object::File::parse(&*data)?;

    let debug_info = match get_line_addresses(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("No debug info: {}", e);
            Default::default()
        }
    };

    let mut text_section = vec![];
    for section in file.sections() {
        let name = match section.name() {
            Ok(s) => s,
            Err(e) => continue
        };
        if name == ".text" {
            let EXAMPLE_CODE_RIP = 0;
            const HEXBYTES_COLUMN_BYTE_LENGTH: usize = 10;
            let bytes = section.data()?;
            text_section = decode_instructions(bytes);
        }
    }
    Ok(ObjectFile {
        debug_info,
        text_section,
    })
}

fn decode_instructions(bytes: &[u8]) -> Vec<DecodedInstruction> {
    let mut decoded_instructions = vec![]; 
    let mut decoder = Decoder::with_ip(64, bytes, 0, DecoderOptions::NONE);

    // Formatters: Masm*, Nasm*, Gas* (AT&T) and Intel* (XED).
    // For fastest code, see `SpecializedFormatter` which is ~3.3x faster. Use it if formatting
    // speed is more important than being able to re-assemble formatted instructions.
    let mut formatter = NasmFormatter::new();

    // Change some options, there are many more
    formatter.options_mut().set_first_operand_char_index(10);

    // String implements FormatterOutput
    let mut output = String::new();

    // Initialize this outside the loop because decode_out() writes to every field
    let mut instruction = Instruction::default();

    // The decoder also implements Iterator/IntoIterator so you could use a for loop:
    //      for instruction in &mut decoder { /* ... */ }
    // or collect():
    //      let instructions: Vec<_> = decoder.into_iter().collect();
    // but can_decode()/decode_out() is a little faster:
    while decoder.can_decode() {
        // There's also a decode() method that returns an instruction but that also
        // means it copies an instruction (40 bytes):
        //     instruction = decoder.decode();
        decoder.decode_out(&mut instruction);

        // Format the instruction ("disassemble" it)
        output.clear();
        formatter.format(&instruction, &mut output);

        // Eg. "00007FFAC46ACDB2 488DAC2400FFFFFF     lea       rbp,[rsp-100h]"
        let instr = output.split_whitespace().collect::<Vec<_>>();
        let operands = if instr.len() > 1 {
            instr[1].split(',').map(|x| x.to_string()).collect::<Vec<String>>()
        } else {
            vec![]
        };
        decoded_instructions.push(DecodedInstruction {
            address: instruction.ip() as _,
            name: instr[0].to_string(),
            operands
        });
    }
    decoded_instructions
}

fn get_addresses_from_program<R, Offset>(
    prog: IncompleteLineProgram<R>,
    debug_strs: &DebugStr<R>,
    result: &mut BTreeMap<usize, Vec<SourceLocation>>,
) -> Result<()>
where
    R: Reader<Offset = Offset>,
    Offset: ReaderOffset,
{
    let get_string = |x: R| x.to_string().map(|y| y.to_string()).ok();
    let (cprog, seq) = prog.sequences()?;
    for s in seq {
        let mut sm = cprog.resume_from(&s);
        while let Ok(Some((header, &ln_row))) = sm.next_row() {
            if ln_row.end_sequence() {
                break;
            }
            // If this row isn't useful move on
            if !ln_row.is_stmt() || ln_row.line().is_none() {
                continue;
            }
            if let Some(file) = ln_row.file(header) {
                let mut path = PathBuf::new();
                if let Some(dir) = file.directory(header) {
                    if let Some(temp) = dir.string_value(debug_strs).and_then(get_string) {
                        path.push(temp);
                    }
                }
                if let Some(file) = file.path_name().string_value(debug_strs).and_then(get_string) {
                    path.push(file);
                    if !path.is_file() {
                        // Not really a source file!
                        continue;
                    }
                    let line = ln_row.line().unwrap();
                    let column = match ln_row.column() {
                        ColumnType::LeftEdge => 1, // Columns aren't zero-indexed
                        ColumnType::Column(nz) => nz.get() as usize,
                    };
                    let address = ln_row.address() as usize;
                    if address > 0 && path.display().to_string().contains("loader.rs") {
                        let loc = SourceLocation {
                            file: path.into(),
                            line: line.get() as usize,
                            column,
                        };
                        result.entry(address).or_default().push(loc);
                    }
                }
            }
        }
    }
    Ok(())
}

fn get_line_addresses<'data>(
    obj: &'data impl object::read::Object<'data>,
) -> anyhow::Result<BTreeMap<usize, Vec<SourceLocation>>> {
    let endian = if obj.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };
    let debug_info = obj.section_by_name(".debug_info").context("No debug_info")?;
    let debug_info = DebugInfo::new(debug_info.data()?, endian);
    let debug_abbrev = obj.section_by_name(".debug_abbrev").context("No debug_abbrev")?;
    let debug_abbrev = DebugAbbrev::new(debug_abbrev.data()?, endian);
    let debug_strings = obj.section_by_name(".debug_str").context("No debug_str")?;
    let debug_strings = DebugStr::new(debug_strings.data()?, endian);
    let debug_line = obj.section_by_name(".debug_line").context("No debug_line")?;
    let debug_line = DebugLine::new(debug_line.data()?, endian);
    
    let mut iter = debug_info.units();
    let mut result = BTreeMap::new();
    while let Ok(Some(cu)) = iter.next() {
        let addr_size = cu.address_size();
        let abbr = match cu.abbreviations(&debug_abbrev) {
            Ok(a) => a,
            _ => continue,
        };
        
        if let Ok(Some((_, root))) = cu.entries(&abbr).next_dfs() {
            let offset = match root.attr_value(DW_AT_stmt_list) {
                Ok(Some(AttributeValue::DebugLineRef(o))) => o,
                _ => continue,
            };
            let prog = debug_line.program(offset, addr_size, None, None)?; // Here?

            if let Err(e) =
                get_addresses_from_program(prog, &debug_strings, &mut result)
            {
                eprintln!("Potential issue reading test addresses {}", e);
            }
        }
    }
    Ok(result)
}
