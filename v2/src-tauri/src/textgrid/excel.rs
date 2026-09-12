//! excel.rs — Utilitaires pour la manipulation de fichiers Excel (.xlsx).
//! Remplace backend/excel_utils.py (openpyxl) en utilisant umya-spreadsheet.
//!
//! Note de vérification : ce fichier n'a pas pu être compilé de bout en bout
//! dans le sandbox utilisé pour cette conversion — son rustc (1.75, installé
//! via apt) est trop ancien pour la chaîne de dépendances "traitement
//! d'image" qu'embarque umya-spreadsheet (nécessaire chez eux pour les
//! images intégrées dans les .xlsx, aucun rapport avec notre usage texte).
//! Chaque appel ci-dessous a en revanche été vérifié directement dans le
//! code source du crate (structs/spreadsheet.rs, structs/worksheet.rs,
//! reader/xlsx.rs, writer/xlsx.rs) : signatures et exemples testés du
//! crate lui-même. Lancez `cargo check` chez vous (toolchain récente) en
//! première étape pour confirmer.

use std::path::Path;
use umya_spreadsheet::{reader, writer, Spreadsheet};

/// Cherche la première feuille vide d'un classeur Excel, ou en crée une nouvelle.
/// Si le fichier n'existe pas, un classeur neuf est créé (avec sa feuille par
/// défaut, qui sera elle-même trouvée "vide" par la recherche ci-dessous).
///
/// Une feuille est considérée vide si elle ne contient aucune ligne/colonne
/// au-delà de A1 et que A1 est vide — équivalent du test openpyxl
/// `ws.max_row == 1 and ws.max_column == 1 and ws["A1"].value is None`.
///
/// Retourne le classeur ouvert ainsi que le nom de la feuille à utiliser ;
/// c'est à l'appelant d'aller chercher cette feuille (`get_sheet_by_name_mut`),
/// d'y écrire, puis d'appeler `save()`.
pub fn get_or_create_empty_sheet(filepath: &str) -> Result<(Spreadsheet, String), String> {
    let path = Path::new(filepath);

    let mut book = if path.exists() {
        reader::xlsx::read(path).map_err(|e| e.to_string())?
    } else {
        umya_spreadsheet::new_file()
    };

    let existing_empty = book.get_sheet_collection().iter().find_map(|sheet| {
        let is_empty = sheet.get_highest_row() <= 1
            && sheet.get_highest_column() <= 1
            && sheet
                .get_cell((1u32, 1u32))
                .map(|c| c.get_value().trim().is_empty())
                .unwrap_or(true);
        if is_empty {
            Some(sheet.get_name().to_string())
        } else {
            None
        }
    });

    if let Some(name) = existing_empty {
        return Ok((book, name));
    }

    // Aucune feuille vide trouvée : on en ajoute une nouvelle.
    let new_index = book.get_sheet_count();
    let new_name = format!("Feuille{}", new_index + 1);
    book.new_sheet(new_name.as_str()).map_err(|e| e.to_string())?;
    Ok((book, new_name))
}

/// Enregistre le classeur sur disque — équivalent de `wb.save(path)` en Python.
pub fn save(book: &Spreadsheet, filepath: &str) -> Result<(), String> {
    writer::xlsx::write(book, filepath).map_err(|e| e.to_string())
}
