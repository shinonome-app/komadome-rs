//! CSV ヘッダー定数。
//!
//! Ruby `shinonome/app/services/csv_creator.rb` の `csv_header` 変数 (L47, L84, L122) と
//! 完全一致させる。CRLF (`\r\n`) で終端する。

/// 未公開作品基本版 (`list_inp_person_all*.zip`)。13 列。
/// Ruby: csv_creator.rb:47 (write_inp)
pub const INP_BASIC: &str = "人物ID,著者名,作品ID,作品名,仮名遣い種別,翻訳者名等,入力者名,校正者名,状態,状態の開始日,底本名,出版社名,入力に使用した版\r\n";

/// 公開作品基本版 (`list_person_all*.zip`)。14 列 (校正に使用した版が増える)。
/// Ruby: csv_creator.rb:84 (write_finished)
pub const FINISHED_BASIC: &str = "人物ID,著者名,作品ID,作品名,仮名遣い種別,翻訳者名等,入力者名,校正者名,状態,状態の開始日,底本名,出版社名,入力に使用した版,校正に使用した版\r\n";

/// 公開作品拡充版 (`list_person_all_extended*.zip`)。55 列。
/// Ruby: csv_creator.rb:122 (write_extended)
pub const FINISHED_EXTENDED: &str = "作品ID,作品名,作品名読み,ソート用読み,副題,副題読み,原題,初出,分類番号,文字遣い種別,作品著作権フラグ,公開日,最終更新日,図書カードURL,人物ID,姓,名,姓読み,名読み,姓読みソート用,名読みソート用,姓ローマ字,名ローマ字,役割フラグ,生年月日,没年月日,人物著作権フラグ,底本名1,底本出版社名1,底本初版発行年1,入力に使用した版1,校正に使用した版1,底本の親本名1,底本の親本出版社名1,底本の親本初版発行年1,底本名2,底本出版社名2,底本初版発行年2,入力に使用した版2,校正に使用した版2,底本の親本名2,底本の親本出版社名2,底本の親本初版発行年2,入力者,校正者,テキストファイルURL,テキストファイル最終更新日,テキストファイル符号化方式,テキストファイル文字集合,テキストファイル修正回数,XHTML/HTMLファイルURL,XHTML/HTMLファイル最終更新日,XHTML/HTMLファイル符号化方式,XHTML/HTMLファイル文字集合,XHTML/HTMLファイル修正回数\r\n";
