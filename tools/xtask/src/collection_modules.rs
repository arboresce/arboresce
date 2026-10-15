//! Selected ARM64 MachO and LLVM23 rawv11 instrumentation compatibility.

use std::collections::BTreeMap;

use crate::collection_inputs::{CollectError, Result};

const BAD: CollectError = CollectError::Conservation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Module {
    names: Vec<u8>,
    records: Vec<u8>,
    counters: u64,
}

fn slice(bytes: &[u8], offset: u64, length: u64) -> Result<&[u8]> {
    let end = offset.checked_add(length).ok_or(BAD)?;
    bytes
        .get(usize::try_from(offset).map_err(|_| BAD)?..usize::try_from(end).map_err(|_| BAD)?)
        .ok_or(BAD)
}

fn word(bytes: &[u8], offset: u64, width: u64) -> Result<u64> {
    let mut value = [0; 8];
    value[..width as usize].copy_from_slice(slice(bytes, offset, width)?);
    Ok(u64::from_le_bytes(value))
}

fn name(bytes: &[u8]) -> &[u8] {
    bytes.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn records(data: &[u8], data_address: i128, counters_address: i128, count: u64) -> Result<Vec<u8>> {
    if !data.len().is_multiple_of(72) {
        return Err(BAD);
    }
    let mut result = Vec::new();
    let mut ranges = Vec::new();
    for (index, record) in data.as_chunks::<72>().0.iter().enumerate() {
        let length = word(record, 56, 4)?.checked_mul(8).ok_or(BAD)?;
        let start = data_address + index as i128 * 72 + word(record, 16, 8)? as i64 as i128
            - counters_address;
        let start = u64::try_from(start).map_err(|_| BAD)?;
        let end = start.checked_add(length).ok_or(BAD)?;
        if length == 0
            || !start.is_multiple_of(8)
            || end > count.checked_mul(8).ok_or(BAD)?
            || record[24..56].iter().any(|byte| *byte != 0)
            || record[60..72].iter().any(|byte| *byte != 0)
        {
            return Err(BAD);
        }
        ranges.push((start, end));
        result.extend_from_slice(&record[..16]);
        result.extend_from_slice(&start.to_le_bytes());
        result.extend_from_slice(&length.to_le_bytes());
    }
    ranges.sort_unstable();
    let mut end = 0;
    for (start, next) in ranges {
        if start != end {
            return Err(BAD);
        }
        end = next;
    }
    if end != count.checked_mul(8).ok_or(BAD)? {
        return Err(BAD);
    }
    Ok(result)
}

/// Observe the complete supported static instrumentation, including unused names.
pub fn object(bytes: &[u8]) -> Result<Module> {
    slice(bytes, 0, 32)?;
    if word(bytes, 0, 4)? != 0xfeedfacf
        || word(bytes, 4, 4)? != 0x0100000c
        || word(bytes, 12, 4)? != 2
    {
        return Err(BAD);
    }
    let commands = word(bytes, 16, 4)?;
    let table_size = word(bytes, 20, 4)?;
    let table = slice(bytes, 32, table_size)?;
    if commands > table_size / 8 {
        return Err(BAD);
    }
    let mut position = 0;
    let mut sections = BTreeMap::new();
    for _ in 0..commands {
        let kind = word(table, position, 4)?;
        let size = word(table, position + 4, 4)?;
        if size < 8 || !size.is_multiple_of(8) {
            return Err(BAD);
        }
        let command = slice(table, position, size)?;
        position = position.checked_add(size).ok_or(BAD)?;
        if kind != 0x19 {
            continue;
        }
        let vm_address = word(command, 24, 8)?;
        let vm_end = vm_address.checked_add(word(command, 32, 8)?).ok_or(BAD)?;
        let file_offset = word(command, 40, 8)?;
        let file_size = word(command, 48, 8)?;
        slice(bytes, file_offset, file_size)?;
        let file_end = file_offset.checked_add(file_size).ok_or(BAD)?;
        let count = word(command, 64, 4)?;
        if 72u64
            .checked_add(count.checked_mul(80).ok_or(BAD)?)
            .ok_or(BAD)?
            != size
        {
            return Err(BAD);
        }
        for index in 0..count {
            let section = slice(command, 72 + index * 80, 80)?;
            let address = word(section, 32, 8)?;
            let length = word(section, 40, 8)?;
            let offset = word(section, 48, 4)?;
            let zero_fill = matches!(word(section, 64, 4)? & 0xff, 1 | 0xc | 0x12);
            if name(&section[16..32]) != name(&command[8..24])
                || address < vm_address
                || address.checked_add(length).ok_or(BAD)? > vm_end
            {
                return Err(BAD);
            }
            let body = if zero_fill {
                &[][..]
            } else {
                if offset < file_offset || offset.checked_add(length).ok_or(BAD)? > file_end {
                    return Err(BAD);
                }
                slice(bytes, offset, length)?
            };
            let key = name(&section[..16]);
            if matches!(
                key,
                b"__llvm_prf_data" | b"__llvm_prf_cnts" | b"__llvm_prf_names"
            ) {
                if zero_fill || sections.insert(key, (address, body)).is_some() {
                    return Err(BAD);
                }
            } else if key.starts_with(b"__llvm_prf_") && length != 0 {
                return Err(BAD);
            }
        }
    }
    if position != table_size {
        return Err(BAD);
    }
    let (data_address, data) = sections.get(b"__llvm_prf_data".as_slice()).ok_or(BAD)?;
    let (counter_address, counters) = sections.get(b"__llvm_prf_cnts".as_slice()).ok_or(BAD)?;
    let (_, names) = sections.get(b"__llvm_prf_names".as_slice()).ok_or(BAD)?;
    if !counters.len().is_multiple_of(8) {
        return Err(BAD);
    }
    let counters = counters.len() as u64 / 8;
    Ok(Module {
        records: records(
            data,
            *data_address as i128,
            *counter_address as i128,
            counters,
        )?,
        names: names.to_vec(),
        counters,
    })
}

/// Validate the complete selected raw layout, excluding only runtime addresses/counts.
pub fn profile(bytes: &[u8]) -> Result<Module> {
    let header = slice(bytes, 0, 152)?;
    if word(header, 0, 8)? != 0xff6c70726f667281
        || word(header, 8, 8)? != 11
        || word(header, 144, 8)? != 2
        || [2, 4, 6, 7, 8, 9, 10, 11, 16, 17]
            .iter()
            .any(|index| word(header, index * 8, 8) != Ok(0))
    {
        return Err(BAD);
    }
    let data_size = word(header, 24, 8)?.checked_mul(72).ok_or(BAD)?;
    let counters = word(header, 40, 8)?;
    let counter_size = counters.checked_mul(8).ok_or(BAD)?;
    let names_offset = 152u64
        .checked_add(data_size)
        .and_then(|value| value.checked_add(counter_size))
        .ok_or(BAD)?;
    let names_size = word(header, 96, 8)?;
    let padding = (8 - names_size % 8) % 8;
    let names = slice(bytes, names_offset, names_size)?;
    let end = names_offset.checked_add(names_size).ok_or(BAD)?;
    if end.checked_add(padding).ok_or(BAD)? != bytes.len() as u64
        || slice(bytes, end, padding)?.iter().any(|byte| *byte != 0)
    {
        return Err(BAD);
    }
    Ok(Module {
        records: records(
            slice(bytes, 152, data_size)?,
            0,
            word(header, 104, 8)? as i64 as i128,
            counters,
        )?,
        names: names.to_vec(),
        counters,
    })
}

/// Metadata compatibility supplements the separately retained executable/PID binding.
pub fn matching(object_bytes: &[u8], profile_bytes: &[u8]) -> Result<()> {
    if object(object_bytes)? != profile(profile_bytes)? {
        return Err(BAD);
    }
    Ok(())
}

pub fn unique(modules: &[Module]) -> Result<()> {
    for (index, module) in modules.iter().enumerate() {
        if modules[..index].contains(module) {
            return Err(BAD);
        }
    }
    Ok(())
}
