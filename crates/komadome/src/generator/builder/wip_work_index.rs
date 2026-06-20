use anyhow::Result;
use serde_json::{Value, json};

use crate::data::models::WipWorkIndexData;
use crate::generator::builder::work_index::build_pagination;
use crate::generator::kana::Kana;

const PAGE_SIZE: usize = 50;

pub fn build_wip_work_index_context(data: &WipWorkIndexData) -> Result<Value> {
    let kana = Kana::from_symbol(&data.kana_symbol);
    let display_char = kana.and_then(|k| k.display_char()).unwrap_or("");

    let works: Vec<Value> = data
        .works
        .iter()
        .enumerate()
        .map(|(idx, w)| {
            let row_number = idx + 1 + ((data.page - 1) * PAGE_SIZE);
            json!({
                "row_number": row_number,
                "id": w.id,
                "title": &w.title,
                "subtitle": w.subtitle.as_deref().unwrap_or(""),
                "kana_type_name": w.kana_type_name.as_deref().unwrap_or(""),
                "author_name": w.author_name.as_deref().unwrap_or(""),
                "author_id": w.author_id,
                "base_author_name": w.base_author_name.as_deref().unwrap_or(""),
                "translator_text": w.translator_text.as_deref().unwrap_or(""),
                "inputer_text": w.inputer_text.as_deref().unwrap_or(""),
                "proofreader_text": w.proofreader_text.as_deref().unwrap_or(""),
                "work_status_name": w.work_status_name.as_deref().unwrap_or(""),
                "started_on": w.started_on.as_deref().unwrap_or(""),
                "teihon_title": w.teihon_title.as_deref().unwrap_or(""),
                "teihon_publisher": w.teihon_publisher.as_deref().unwrap_or(""),
                "teihon_input_edition": w.teihon_input_edition.as_deref().unwrap_or(""),
            })
        })
        .collect();

    Ok(json!({
        "page_title": format!("作業中　作品一覧：{} | 青空文庫", display_char),
        "bgcolor": crate::tailwind::bgcolor::DEFAULT,
        "kana_symbol": data.kana_symbol,
        "kana_display": display_char,
        "page": data.page,
        "total_pages": data.total_pages,
        "has_pagination": data.total_pages > 1,
        "prev_page": if data.page > 1 { Some(data.page - 1) } else { None },
        "next_page": if data.page < data.total_pages { Some(data.page + 1) } else { None },
        "works": works,
        "pagination": build_pagination(data.page, data.total_pages),
    }))
}

pub fn wip_work_index_filename(kana_symbol: &str, page: usize) -> String {
    format!("sakuhin_inp_{kana_symbol}{page}.html")
}
