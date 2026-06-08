use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load all records from a JSONL file
pub fn load_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open JSONL file: {}", path.display()))?;

    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_num, line) in reader.lines().enumerate() {
        let line = line.with_context(|| {
            format!("Failed to read line {} of {}", line_num + 1, path.display())
        })?;

        if line.trim().is_empty() {
            continue;
        }

        let record: T = serde_json::from_str(&line).with_context(|| {
            format!(
                "Failed to parse JSON at line {} of {}",
                line_num + 1,
                path.display()
            )
        })?;

        records.push(record);
    }

    Ok(records)
}

/// Iterator over JSONL records (for streaming large files)
pub struct JsonlIterator<T> {
    reader: BufReader<File>,
    line_buffer: String,
    line_num: usize,
    path: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: DeserializeOwned> JsonlIterator<T> {
    pub fn new(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open JSONL file: {}", path.display()))?;

        Ok(Self {
            reader: BufReader::new(file),
            line_buffer: String::new(),
            line_num: 0,
            path: path.display().to_string(),
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<T: DeserializeOwned> Iterator for JsonlIterator<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.line_buffer.clear();
            self.line_num += 1;

            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    if self.line_buffer.trim().is_empty() {
                        continue;
                    }

                    let result = serde_json::from_str(&self.line_buffer).with_context(|| {
                        format!(
                            "Failed to parse JSON at line {} of {}",
                            self.line_num, self.path
                        )
                    });

                    return Some(result);
                }
                Err(e) => {
                    return Some(Err(anyhow::Error::new(e).context(format!(
                        "Failed to read line {} of {}",
                        self.line_num, self.path
                    ))));
                }
            }
        }
    }
}

/// Count lines in a JSONL file
pub fn count_jsonl_lines(path: &Path) -> Result<usize> {
    let file =
        File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let reader = BufReader::new(file);
    let count = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.trim().is_empty())
        .count();

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct TestRecord {
        id: i64,
        name: String,
    }

    #[test]
    fn test_load_jsonl() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"id": 1, "name": "Alice"}}"#).unwrap();
        writeln!(file, r#"{{"id": 2, "name": "Bob"}}"#).unwrap();

        let records: Vec<TestRecord> = load_jsonl(file.path()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Alice");
    }

    #[test]
    fn test_jsonl_iterator() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"id": 1, "name": "Alice"}}"#).unwrap();
        writeln!(file, r#"{{"id": 2, "name": "Bob"}}"#).unwrap();

        let iter: JsonlIterator<TestRecord> = JsonlIterator::new(file.path()).unwrap();
        let records: Vec<TestRecord> = iter.map(|r| r.unwrap()).collect();

        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_count_jsonl_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"id": 1}}"#).unwrap();
        writeln!(file).unwrap(); // empty line
        writeln!(file, r#"{{"id": 2}}"#).unwrap();

        let count = count_jsonl_lines(file.path()).unwrap();
        assert_eq!(count, 2);
    }
}
