/// Pagination utilities
///
/// Generates pagination data similar to Ruby's Pagy gem

#[derive(Debug, Clone)]
pub struct Pagination {
    pub current_page: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub page_size: usize,
    pub prev_page: Option<usize>,
    pub next_page: Option<usize>,
    pub series: Vec<PageItem>,
}

#[derive(Debug, Clone)]
pub enum PageItem {
    Number(usize),
    Current(usize),
    Gap,
}

impl Pagination {
    pub fn new(current_page: usize, total_items: usize, page_size: usize) -> Self {
        let total_pages = (total_items + page_size - 1) / page_size;
        let total_pages = total_pages.max(1);
        let current_page = current_page.clamp(1, total_pages);

        let prev_page = if current_page > 1 {
            Some(current_page - 1)
        } else {
            None
        };

        let next_page = if current_page < total_pages {
            Some(current_page + 1)
        } else {
            None
        };

        let series = Self::build_series(current_page, total_pages);

        Self {
            current_page,
            total_pages,
            total_items,
            page_size,
            prev_page,
            next_page,
            series,
        }
    }

    fn build_series(current: usize, total: usize) -> Vec<PageItem> {
        if total <= 7 {
            // Show all pages
            return (1..=total)
                .map(|p| {
                    if p == current {
                        PageItem::Current(p)
                    } else {
                        PageItem::Number(p)
                    }
                })
                .collect();
        }

        let mut items = Vec::new();

        // Always show first page
        if current == 1 {
            items.push(PageItem::Current(1));
        } else {
            items.push(PageItem::Number(1));
        }

        // Gap or pages near start
        if current > 4 {
            items.push(PageItem::Gap);
        } else {
            for p in 2..current {
                items.push(PageItem::Number(p));
            }
        }

        // Current page and neighbors
        if current > 1 && current < total {
            let start = if current > 4 { current - 1 } else { current };
            let end = if current < total - 3 {
                current + 1
            } else {
                current
            };

            for p in start..=end {
                if p == current {
                    items.push(PageItem::Current(p));
                } else if p > 1 && p < total {
                    items.push(PageItem::Number(p));
                }
            }
        }

        // Gap or pages near end
        if current < total - 3 {
            items.push(PageItem::Gap);
        } else {
            for p in (current + 1)..total {
                items.push(PageItem::Number(p));
            }
        }

        // Always show last page
        if current == total {
            items.push(PageItem::Current(total));
        } else {
            items.push(PageItem::Number(total));
        }

        items
    }

    /// Get items for the current page from a slice
    pub fn slice_items<'a, T>(&self, items: &'a [T]) -> &'a [T] {
        let start = (self.current_page - 1) * self.page_size;
        let end = (start + self.page_size).min(items.len());

        if start >= items.len() {
            &[]
        } else {
            &items[start..end]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_single_page() {
        let p = Pagination::new(1, 25, 50);
        assert_eq!(p.current_page, 1);
        assert_eq!(p.total_pages, 1);
        assert!(p.prev_page.is_none());
        assert!(p.next_page.is_none());
    }

    #[test]
    fn test_pagination_multiple_pages() {
        let p = Pagination::new(2, 150, 50);
        assert_eq!(p.current_page, 2);
        assert_eq!(p.total_pages, 3);
        assert_eq!(p.prev_page, Some(1));
        assert_eq!(p.next_page, Some(3));
    }

    #[test]
    fn test_slice_items() {
        let items: Vec<i32> = (1..=100).collect();
        let p = Pagination::new(2, 100, 10);
        let slice = p.slice_items(&items);
        assert_eq!(slice, &[11, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
    }
}
