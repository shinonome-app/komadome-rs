use anyhow::Result;
use serde_json::{Value, json};

use crate::data::models::ListInpData;
use crate::generator::builder::work_index::build_pagination;

const PAGE_SIZE: usize = 50;

pub fn build_list_inp_context(data: &ListInpData) -> Result<Value> {
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
        "page_title": format!("作業中　作家別作品一覧：{} | 青空文庫", data.person_name),
        "bgcolor": "bg-sky-50",
        "person_id": data.person_id,
        "person_name": data.person_name,
        "page": data.page,
        "total_pages": data.total_pages,
        "has_pagination": data.total_pages > 1,
        "prev_page": if data.page > 1 { Some(data.page - 1) } else { None },
        "next_page": if data.page < data.total_pages { Some(data.page + 1) } else { None },
        "works": works,
        "pagination": build_pagination(data.page, data.total_pages),
    }))
}

pub fn list_inp_filename(person_id: i64, page: usize) -> String {
    format!("list_inp{person_id}_{page}.html")
}
