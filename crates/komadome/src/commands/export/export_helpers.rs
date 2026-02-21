use anyhow::Result;
use serde::Serialize;
use std::io::Write;

pub const PAGE_SIZE: usize = 50;

pub fn calculate_total_pages(total_items: usize) -> usize {
    let pages = (total_items as f64 / PAGE_SIZE as f64).ceil() as usize;
    if pages == 0 { 1 } else { pages }
}

pub fn write_jsonl_line<T: Serialize>(writer: &mut impl Write, data: &T) -> Result<()> {
    serde_json::to_writer(&mut *writer, data)?;
    writer.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total_pages_zero() {
        assert_eq!(calculate_total_pages(0), 1);
    }

    #[test]
    fn test_calculate_total_pages_exact_page() {
        assert_eq!(calculate_total_pages(50), 1);
    }

    #[test]
    fn test_calculate_total_pages_one_over() {
        assert_eq!(calculate_total_pages(51), 2);
    }

    #[test]
    fn test_calculate_total_pages_two_pages() {
        assert_eq!(calculate_total_pages(100), 2);
    }

    #[test]
    fn test_calculate_total_pages_three_pages() {
        assert_eq!(calculate_total_pages(101), 3);
    }

    #[test]
    fn test_write_jsonl_line() {
        let mut buf = Vec::new();
        write_jsonl_line(&mut buf, &serde_json::json!({"key": "value"})).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "{\"key\":\"value\"}\n");
    }
}
