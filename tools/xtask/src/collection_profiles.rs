//! Complete selected LLVM Front-end profile inspection, including zero activity.

use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::collection_inputs::{CollectError, MAX_PROTOCOL_BYTES, Result};

const BAD: CollectError = CollectError::Conservation;

fn decimal(line: &str, prefix: &str) -> Result<u64> {
    let value = line.strip_prefix(prefix).ok_or(BAD)?;
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(BAD);
    }
    value.parse().map_err(|_| BAD)
}

#[derive(Default)]
struct Counts {
    functions: u64,
    positive: u64,
    max_entry: u64,
    max_block: u64,
    counters: u64,
    total: u64,
}

impl Counts {
    fn function(&mut self, lines: &[&str]) -> Result<()> {
        let [_, hash, counters, entry, blocks] = lines else {
            return Err(BAD);
        };
        let hash = hash.strip_prefix("    Hash: 0x").ok_or(BAD)?;
        if hash.is_empty() || hash.len() > 16 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(BAD);
        }
        let counters = decimal(counters, "    Counters: ")?;
        let entry = decimal(entry, "    Function count: ")?;
        let block_text = blocks
            .strip_prefix("    Block counts: [")
            .and_then(|s| s.strip_suffix(']'))
            .ok_or(BAD)?;
        let mut block_counts = Vec::new();
        if !block_text.is_empty() {
            for block in block_text.split(", ") {
                block_counts.push(decimal(block, "")?);
            }
        }
        if counters == 0 || counters != block_counts.len() as u64 + 1 {
            return Err(BAD);
        }
        self.functions = self.functions.checked_add(1).ok_or(BAD)?;
        self.positive += u64::from(entry > 0 || block_counts.iter().any(|count| *count > 0));
        self.max_entry = self.max_entry.max(entry);
        self.counters = self.counters.checked_add(counters).ok_or(BAD)?;
        self.total = self.total.checked_add(entry).ok_or(BAD)?;
        for block in block_counts {
            self.max_block = self.max_block.max(block);
            self.total = self.total.checked_add(block).ok_or(BAD)?;
        }
        Ok(())
    }
}

/// Parse every function and summary, rather than matching a successful prefix.
pub fn inspect(raw: &str) -> Result<Value> {
    if raw.len() > MAX_PROTOCOL_BYTES || !raw.ends_with('\n') {
        return Err(BAD);
    }
    let lines = raw.split_terminator('\n').collect::<Vec<_>>();
    let mut remaining = lines.as_slice();
    let mut counts = Counts::default();
    let mut names = BTreeSet::new();
    if remaining.first() == Some(&"Counters:") {
        remaining = &remaining[1..];
        while remaining.first().is_some_and(|line| line.starts_with("  ")) {
            let name = remaining[0]
                .strip_prefix("  ")
                .and_then(|s| s.strip_suffix(':'))
                .ok_or(BAD)?;
            if name.is_empty() || !names.insert(name) || remaining.len() < 5 {
                return Err(BAD);
            }
            counts.function(&remaining[..5])?;
            remaining = &remaining[5..];
        }
    }
    let [level, shown, total, max_entry, max_block, counters, sum] = remaining else {
        return Err(BAD);
    };
    if *level != "Instrumentation level: Front-end"
        || decimal(shown, "Functions shown: ")? != counts.functions
        || decimal(total, "Total functions: ")? != counts.functions
        || decimal(max_entry, "Maximum function count: ")? != counts.max_entry
        || decimal(max_block, "Maximum internal block count: ")? != counts.max_block
        || decimal(counters, "Total number of blocks: ")? != counts.counters
        || decimal(sum, "Total count: ")? != counts.total
    {
        return Err(BAD);
    }
    Ok(json!({"functions":counts.functions,"positive_functions":counts.positive}))
}
