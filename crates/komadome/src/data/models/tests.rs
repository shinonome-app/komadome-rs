use super::*;

fn fixture_path(name: &str) -> String {
    format!(
        "{}/tests/fixtures/{}",
        env!("CARGO_MANIFEST_DIR"),
        name
    )
}

fn load_fixture<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let path = fixture_path(name);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", name, e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", name, e))
}

fn roundtrip_check<T: serde::de::DeserializeOwned>(name: &str) -> (T, serde_json::Value) {
    let path = fixture_path(name);
    let content = std::fs::read_to_string(&path).unwrap();
    let original_value: serde_json::Value = serde_json::from_str(&content).unwrap();
    let parsed: T = serde_json::from_str(&content).unwrap();
    (parsed, original_value)
}

#[test]
fn test_card_data_deserialization() {
    let (card, _original): (CardData, _) = roundtrip_check("card_data.json");
    assert_eq!(card.work_id, 12345);
    assert_eq!(card.person_id, 100);
    assert_eq!(card.title, "Test Work Title");
    assert_eq!(card.title_kana.as_deref(), Some("テストサクヒンタイトル"));
    assert_eq!(card.subtitle.as_deref(), Some("A Subtitle"));
    assert_eq!(card.kana_type.as_deref(), Some("新字新仮名"));
    assert_eq!(card.authors.len(), 1);
    assert_eq!(card.authors[0].id, 100);
    assert_eq!(card.authors[0].copyright_flag, false);
    assert_eq!(card.translators.len(), 1);
    assert_eq!(card.editors.len(), 0);
    assert_eq!(card.workfiles.len(), 1);
    assert_eq!(card.workfiles[0].filetype_id, 1);
    assert_eq!(card.workfiles[0].is_html, false);
    assert_eq!(card.original_books.len(), 1);
    assert_eq!(card.work_workers.len(), 2);
    assert_eq!(card.bibclasses.len(), 1);
    assert_eq!(card.sites.len(), 1);
    assert_eq!(card.work_people_details.len(), 1);
    assert_eq!(card.work_people_details[0].role_name, "著者");
}

#[test]
fn test_card_data_methods() {
    let card: CardData = load_fixture("card_data.json");
    assert_eq!(card.has_copyright(), false);
    assert_eq!(card.primary_author().unwrap().id, 100);
    assert_eq!(card.card_path(), "cards/000100/card12345.html");
}

#[test]
fn test_person_page_data_deserialization() {
    let (data, _original): (PersonPageData, _) = roundtrip_check("person_page_data.json");
    assert_eq!(data.person.id, 100);
    assert_eq!(data.person.last_name, "Author");
    assert_eq!(data.person.first_name.as_deref(), Some("Name"));
    assert_eq!(data.person.full_name(), "Author Name");
    assert_eq!(data.person.full_name_kana(), "チョシャ メイ");
    assert_eq!(data.person.copyright_flag, false);
    assert_eq!(data.works.len(), 1);
    assert_eq!(data.works[0].id, 12345);
    assert_eq!(data.works[0].role_id, 1);
    assert_eq!(data.works[0].work_people.len(), 1);
    assert_eq!(data.works[0].work_people[0].person_id, 200);
    assert_eq!(data.unpublished_works.len(), 0);
    assert_eq!(data.sites.len(), 1);
    assert_eq!(data.other_base_people.len(), 1);
    assert_eq!(data.other_base_people[0].id, 101);
}

#[test]
fn test_work_index_data_deserialization() {
    let (data, _original): (WorkIndexData, _) = roundtrip_check("work_index_data.json");
    assert_eq!(data.kana_symbol, "a");
    assert_eq!(data.page, 1);
    assert_eq!(data.total_pages, 3);
    assert_eq!(data.works.len(), 1);
    assert_eq!(data.works[0].id, 100);
    assert_eq!(data.works[0].person_id, Some(50));
    assert_eq!(data.works[0].card_person_id.as_deref(), Some("000050"));
}

#[test]
fn test_person_index_data_deserialization() {
    let (data, _original): (PersonIndexData, _) = roundtrip_check("person_index_data.json");
    assert_eq!(data.kana_column, "a");
    assert_eq!(data.column_display, "あ");
    assert_eq!(data.sections.len(), 1);
    assert_eq!(data.sections[0].kana_char, "あ");
    assert_eq!(data.sections[0].section_index, 1);
    assert_eq!(data.sections[0].people.len(), 1);
    assert_eq!(data.sections[0].people[0].work_count, 5);
}

#[test]
fn test_whatsnew_data_deserialization() {
    let (data, _original): (WhatsnewData, _) = roundtrip_check("whatsnew_data.json");
    assert_eq!(data.year, None);
    assert_eq!(data.page, 1);
    assert_eq!(data.total_pages, 2);
    assert_eq!(data.entries.len(), 1);
    assert_eq!(data.entries[0].work_id, 999);
    assert_eq!(data.entries[0].card_person_id, Some(100));
    assert_eq!(data.entries[0].started_on.as_deref(), Some("2024-06-01"));
}

#[test]
fn test_top_page_data_deserialization() {
    let (data, _original): (TopPageData, _) = roundtrip_check("top_page_data.json");
    assert_eq!(data.new_works.len(), 1);
    assert_eq!(data.new_works[0].work_id, 555);
    assert_eq!(data.new_works[0].card_person_id, Some(200));
    assert_eq!(data.topics.len(), 1);
    assert_eq!(data.topics[0].year, Some(2024));
    assert_eq!(data.works_count, 18000);
    assert_eq!(data.works_copyright_count, 3000);
    assert_eq!(data.works_noncopyright_count, 15000);
}

#[test]
fn test_wip_work_index_data_deserialization() {
    let (data, _original): (WipWorkIndexData, _) = roundtrip_check("wip_work_index_data.json");
    assert_eq!(data.kana_symbol, "ka");
    assert_eq!(data.page, 1);
    assert_eq!(data.total_pages, 1);
    assert_eq!(data.works.len(), 1);
    assert_eq!(data.works[0].id, 777);
    assert_eq!(data.works[0].author_id, Some(300));
    assert_eq!(data.works[0].teihon_title.as_deref(), Some("Teihon Book"));
}

#[test]
fn test_wip_person_index_data_deserialization() {
    let (data, _original): (WipPersonIndexData, _) = roundtrip_check("wip_person_index_data.json");
    assert_eq!(data.kana_column, "ka");
    assert_eq!(data.column_display, "か");
    assert_eq!(data.sections.len(), 1);
    assert_eq!(data.sections[0].people.len(), 1);
    assert_eq!(data.sections[0].people[0].unpublished_count, 3);
    assert_eq!(data.sections[0].people[0].copyright_flag, true);
}

#[test]
fn test_person_all_index_data_deserialization() {
    let (data, _original): (PersonAllIndexData, _) = roundtrip_check("person_all_index_data.json");
    assert_eq!(data.kana_column, "a");
    assert_eq!(data.sections.len(), 1);
    assert_eq!(data.sections[0].people.len(), 1);
    let person = &data.sections[0].people[0];
    assert_eq!(person.published_count, 5);
    assert_eq!(person.unpublished_count, 2);
    assert_eq!(person.total_count, 7);
}

#[test]
fn test_list_inp_data_deserialization() {
    let (data, _original): (ListInpData, _) = roundtrip_check("list_inp_data.json");
    assert_eq!(data.person_id, 300);
    assert_eq!(data.person_name, "WIP Person");
    assert_eq!(data.page, 1);
    assert_eq!(data.works.len(), 1);
    assert_eq!(data.works[0].id, 888);
    assert_eq!(data.works[0].work_status_name.as_deref(), Some("校正中"));
}

#[test]
fn test_news_data_deserialization() {
    let (data, _original): (NewsData, _) = roundtrip_check("news_data.json");
    assert_eq!(data.year, 2024);
    assert_eq!(data.entries.len(), 1);
    assert_eq!(data.entries[0].id, 42);
    assert_eq!(data.entries[0].flag, true);
    assert_eq!(data.entries[0].published_on.as_deref(), Some("2024-03-15"));
}
