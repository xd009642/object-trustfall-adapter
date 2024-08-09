use crate::adapter::SourceLocation;
use anyhow::Context;
use gimli::*;
use object::{read::ObjectSection, Object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DecodedInstruction {
    pub address: u64,
    pub name: String,
    pub operands: Vec<String>,
    pub length: usize,
}

pub(crate) fn get_addresses_from_program<R, Offset>(
    prog: IncompleteLineProgram<R>,
    debug_strs: &DebugStr<R>,
    result: &mut BTreeMap<u64, Vec<Rc<SourceLocation>>>,
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
                if let Some(file) = file
                    .path_name()
                    .string_value(debug_strs)
                    .and_then(get_string)
                {
                    path.push(file);
                    let line = ln_row.line().unwrap();
                    let column = match ln_row.column() {
                        ColumnType::LeftEdge => 1, // Columns aren't zero-indexed
                        ColumnType::Column(nz) => nz.get() as usize,
                    };
                    let address = ln_row.address();
                    if address > 0 {
                        let loc = SourceLocation {
                            file: path.into(),
                            line: line.get() as usize,
                            column,
                        };
                        result.entry(address).or_default().push(loc.into());
                    }
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn get_line_addresses<'data>(
    obj: &'data impl object::read::Object<'data>,
) -> anyhow::Result<BTreeMap<u64, Vec<Rc<SourceLocation>>>> {
    let endian = if obj.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };
    let debug_info = obj
        .section_by_name(".debug_info")
        .context("No debug_info")?;
    let debug_info = DebugInfo::new(debug_info.data()?, endian);
    let debug_abbrev = obj
        .section_by_name(".debug_abbrev")
        .context("No debug_abbrev")?;
    let debug_abbrev = DebugAbbrev::new(debug_abbrev.data()?, endian);
    let debug_strings = obj.section_by_name(".debug_str").context("No debug_str")?;
    let debug_strings = DebugStr::new(debug_strings.data()?, endian);
    let debug_line = obj
        .section_by_name(".debug_line")
        .context("No debug_line")?;
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

            if let Err(e) = get_addresses_from_program(prog, &debug_strings, &mut result) {
                eprintln!("Potential issue reading test addresses {}", e);
            }
        }
    }
    Ok(result)
}
