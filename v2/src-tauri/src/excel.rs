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
use serde::Serialize;
use umya_spreadsheet::{reader, writer, Spreadsheet};

/// Récupère (en créant si besoin) une feuille prête à recevoir des données,
/// nommée `"{base_title}_N"` où N est le premier entier (à partir de 1) tel
/// que ce nom n'existe pas encore dans le classeur — ex: `Concordancier_1`,
/// puis `Concordancier_2` au prochain export, etc. Aucun export précédent
/// n'est jamais écrasé ni supprimé.
///
/// Si le fichier n'existe pas du tout, un classeur neuf est créé.
///
/// Retourne le classeur ouvert ainsi que le nom réel de la feuille créée
/// (à réutiliser tel quel pour l'appel à `write_table`).
pub fn get_or_create_sheet(filepath: &str, base_title: &str) -> Result<(Spreadsheet, String), String> {
    let path = Path::new(filepath);

    let mut book = if path.exists() {
        reader::xlsx::read(path).map_err(|e| e.to_string())?
    } else {
        umya_spreadsheet::new_file()
    };

    // Premier nom "base_title_N" encore libre dans le classeur.
    let mut n: u32 = 1;
    let final_name = loop {
        let candidate = format!("{}_{}", base_title, n);
        if book.get_sheet_by_name(&candidate).is_none() {
            break candidate;
        }
        n += 1;
    };

    // Réutilise une feuille vide existante (ex: la feuille par défaut d'un
    // classeur tout neuf) plutôt que d'en empiler des inutiles ; sinon en
    // crée une nouvelle.
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

    let working_name = if let Some(name) = existing_empty {
        name
    } else {
        let new_index = book.get_sheet_count();
        let new_name = format!("Feuille{}", new_index + 1);
        book.new_sheet(new_name.as_str()).map_err(|e| e.to_string())?;
        new_name
    };

    if let Some(sheet) = book.get_sheet_by_name_mut(&working_name) {
        sheet.set_name(final_name.clone());
    }

    Ok((book, final_name))
}

/// Enregistre le classeur sur disque — équivalent de `wb.save(path)` en Python.
pub fn save(book: &Spreadsheet, filepath: &str) -> Result<(), String> {
    writer::xlsx::write(book, filepath).map_err(|e| e.to_string())
}

/// Écrit un en-tête + des lignes de données (chaînes) dans la feuille nommée
/// `sheet_name` du classeur, à partir de la cellule A1. Toutes les valeurs
/// sont écrites comme du texte (pas de détection numérique) : simple et
/// fiable, quitte à perdre le typage natif "nombre" d'Excel pour les
/// colonnes comme x_min/x_max/ID. Amélioration possible plus tard si
/// nécessaire.
pub fn write_table(
    book: &mut Spreadsheet,
    sheet_name: &str,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<(), String> {
    let sheet = book
        .get_sheet_by_name_mut(sheet_name)
        .ok_or_else(|| format!("Feuille '{}' introuvable", sheet_name))?;

    for (col_idx, header) in headers.iter().enumerate() {
        sheet
            .get_cell_mut(((col_idx + 1) as u32, 1u32))
            .set_value(header.clone());
    }

    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, value) in row.iter().enumerate() {
            sheet
                .get_cell_mut(((col_idx + 1) as u32, (row_idx + 2) as u32))
                .set_value(value.clone());
        }
    }

    Ok(())
}
