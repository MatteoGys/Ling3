"""
onglet_3_view.py — Vue de Tokenisation : Analyse des occurrences de mots (TextGrid).
"""

import os
import re
from collections import Counter
import flet as ft
from ui.theme import C

# Import de notre fonction utilitaire de gestion Excel (à placer dans backend/excel_utils.py)
try:
    from backend.excel_utils import get_or_create_empty_sheet
except ImportError:
    pass  # Géré dans la méthode d'export pour afficher une erreur propre à l'utilisateur


class Tokenisation_class:
    """Vue permettant de lister, filtrer, trier et exporter les occurrences des mots d'un TextGrid."""

    def __init__(self, page: ft.Page, settings_manager=None) -> None:
        self.page = page
        self.sm = settings_manager

        # ── État des données ──────────────────────────────────────────────────
        self.tiers_data: dict[str, Counter] = {}
        self.global_counts: Counter = Counter()
        self.active_tiers: list[str] = []

        self.sort_col: str = "Tokens"
        self.col_directions: dict[str, bool] = {
            "Tokens": True,
            "Occ. Tot.": True,
        }  # True = ASC (▲/△), False = DESC (▼/▽)
        self.search_query: str = ""

        # ── Composants UI ─────────────────────────────────────────────────────
        self._btn_load = ft.Button(
            "Rafraîchir",
            icon=ft.Icons.REFRESH,
            on_click=self._load_data,
            style=ft.ButtonStyle(
                bgcolor={
                    ft.ControlState.DEFAULT: C["validation_btn_default"],
                    ft.ControlState.HOVERED: C["validation_btn_hovered"],
                },
                color=C["white"],
                shape=ft.RoundedRectangleBorder(radius=8),
            ),
            height=40,
        )

        self._search_field = ft.TextField(
            label="Rechercher un token",
            hint_text="Tapez les premières lettres (ex: s)...",
            on_change=self._on_search,
            bgcolor=C["white"],
            border_color=C["bg_container_border"],
            focused_border_color=C["validation_btn_default"],
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
            prefix_icon=ft.Icons.SEARCH,
            height=40,
            content_padding=ft.Padding.symmetric(horizontal=10, vertical=0),
            expand=True,
        )

        self._tiers_menu = ft.PopupMenuButton(
            bgcolor=C["white"],
            content=ft.Container(
                content=ft.Row(
                    [
                        ft.Text("Tiers", color=C["main_text"], weight=ft.FontWeight.BOLD, size=14),
                        ft.Icon(ft.Icons.ARROW_DROP_DOWN, color=C["main_text"], size=20),
                    ],
                    spacing=4,
                    alignment=ft.MainAxisAlignment.CENTER,
                ),
                bgcolor=C["white"],
                border=ft.Border.all(1, C["bg_container_border"]),
                border_radius=8,
                padding=ft.Padding.symmetric(horizontal=12, vertical=8),
            ),
            items=[],
            tooltip="Sélectionner les Tiers à afficher",
        )

        self._table = ft.DataTable(
            columns=[],
            rows=[],
            bgcolor=C["white"],
            border=ft.Border.all(1, C["bg_container_border"]),
            border_radius=8,
            heading_row_color=C["bg_container"],
            heading_text_style=ft.TextStyle(weight=ft.FontWeight.BOLD, color=C["main_text"]),
            data_text_style=ft.TextStyle(color=C["main_text"]),
        )

        self._status_text = ft.Text("", color=C["secondary_text"], size=13)

        self._btn_export = ft.Button(
            "Export Excel",
            icon=ft.Icons.SAVE_ALT,
            on_click=self._export_excel,
            style=ft.ButtonStyle(
                bgcolor={
                    ft.ControlState.DEFAULT: C["export_btn_default"],
                    ft.ControlState.HOVERED: C["export_btn_hovered"],
                },
                color=C["white"],
                shape=ft.RoundedRectangleBorder(radius=8),
            ),
            height=40,
        )

        # Structure racine
        self.root = ft.Container(
            expand=True,
            bgcolor=C["bg_onglet"],
            padding=28,
            content=ft.Column(
                expand=True,
                spacing=16,
                controls=[
                    ft.Text("Tokenisation & Occurrences", size=26, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Divider(height=1, color=C["divider_color"]),

                    # Barre de contrôles compacte (Rafraîchir, Recherche, Menu Tiers)
                    ft.Container(
                        bgcolor=C["bg_container"],
                        padding=12,
                        border_radius=12,
                        border=ft.Border.all(1, color=C["bg_container_border"]),
                        content=ft.Row(
                            controls=[
                                self._search_field,
                                self._tiers_menu,
                                self._btn_load,
                            ],
                            spacing=12,
                            alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
                            vertical_alignment=ft.CrossAxisAlignment.CENTER,
                        )
                    ),

                    # Tableau de données dans une zone défilable
                    ft.Container(
                        expand=True,
                        bgcolor=C["white"],
                        border_radius=12,
                        border=ft.Border.all(1, color=C["bg_container_border"]),
                        content=ft.ListView(
                            expand=True,
                            controls=[self._table],
                            padding=10
                        )
                    ),

                    # Footer: Statut à gauche, Bouton d'export à droite
                    ft.Row(
                        alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
                        controls=[
                            self._status_text,
                            self._btn_export
                        ]
                    )
                ]
            )
        )

        # Initialisation systématique des colonnes par défaut pour éviter l'erreur Flet
        self._update_table()

    # ── Moteur d'Analyse ─────────────────────────────────────────────────────

    def _read_text_auto(self, filepath: str) -> str:
        """Détecte l'encodage et lit le fichier textuel de manière sécurisée."""
        with open(filepath, 'rb') as f:
            raw = f.read()
        if raw.startswith(b'\xff\xfe') or raw.startswith(b'\xfe\xff'):
            return raw.decode('utf-16')
        if raw.startswith(b'\xef\xbb\xbf'):
            return raw.decode('utf-8-sig')
        try:
            return raw.decode('utf-8')
        except UnicodeDecodeError:
            return raw.decode('utf-16')

    def _load_data(self, _e=None) -> None:
        """Parse les fichiers TextGrid et génère les décomptes par Tiers,
        cumulés sur l'ensemble des fichiers sélectionnés."""
        data = self.sm.get_current_data() if self.sm else {}
        tg_paths = data.get("textgrid_path_files", "")

        # Réinitialisation — UNE SEULE FOIS, avant la boucle, sinon chaque
        # nouveau fichier efface le travail des précédents.
        self.tiers_data.clear()
        self.global_counts.clear()
        self.active_tiers.clear()

        if not tg_paths:
            self._status_text.value = "Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres."
            self._status_text.color = C["red"]
            self._update_table()
            return

        tier_idx_pat = re.compile(r"item\s*\[\s*(\d+)\s*\]\s*:")
        name_pat = re.compile(r'name\s*=\s*"([^"]+)"')
        text_pat = re.compile(r'text\s*=\s*"([^"]*)"')

        missing_files = []
        error_files = []
        processed_count = 0

        for element in tg_paths:
            if not element or not os.path.exists(element):
                # On ne bloque plus tout le lot pour un seul chemin invalide :
                # on le note et on continue avec les fichiers suivants.
                missing_files.append(element or "(chemin vide)")
                continue

            try:
                content = self._read_text_auto(element)
                lines = content.splitlines()
                current_tier = None

                # Parsing structurel (inchangé)
                for line in lines:
                    stripped = line.strip()

                    if tier_idx_pat.search(stripped):
                        current_tier = "Unknown"
                        continue

                    if current_tier is not None:
                        n_match = name_pat.search(stripped)
                        if n_match and current_tier == "Unknown":
                            current_tier = n_match.group(1)
                            if current_tier not in self.tiers_data:
                                self.tiers_data[current_tier] = Counter()
                            continue

                        t_match = text_pat.search(stripped)
                        if t_match and current_tier != "Unknown":
                            txt = t_match.group(1)
                            if txt.strip():
                                tokens = [w.lower() for w in re.split(r"[ ']+", txt) if w]
                                # .update() incrémente les tokens déjà connus
                                # (y compris ceux comptés dans un fichier
                                # précédent) et crée les nouveaux — c'est déjà
                                # le comportement que tu cherchais à coder.
                                self.tiers_data[current_tier].update(tokens)

                processed_count += 1

            except Exception as ex:
                error_files.append(f"{os.path.basename(element)} ({ex})")

        # Agrégation globale — calculée UNE SEULE FOIS, à la fin, sur l'état
        # cumulé de tous les fichiers traités (sinon on recompte en double
        # les fichiers précédents à chaque itération).
        for tier_counts in self.tiers_data.values():
            self.global_counts.update(tier_counts)

        self._build_tier_menu()

        if missing_files or error_files:
            parts = [f"{processed_count} fichier(s) analysé(s)"]
            if missing_files:
                parts.append(f"{len(missing_files)} introuvable(s)")
            if error_files:
                parts.append(f"{len(error_files)} en erreur")
            self._status_text.value = " · ".join(parts) + "."
            self._status_text.color = C["red"]
        else:
            self._status_text.value = ""
            self._status_text.color = C["export_btn_default"]

        self._update_table()

    # ── Moteur d'Affichage et Tri ────────────────────────────────────────────

    def _build_tier_menu(self):
        """Génère les options du menu déroulant pour activer/désactiver les tiers."""
        self._tiers_menu.items.clear()
        for tier_name in self.tiers_data.keys():
            is_checked = tier_name in self.active_tiers
            item = ft.PopupMenuItem(
                content=ft.Text(
                    tier_name,
                    color=C["main_text"],
                ),
                checked=is_checked,
                on_click=lambda e, t=tier_name: self._toggle_tier(t)
            )
            self._tiers_menu.items.append(item)

    def _toggle_tier(self, tier_name: str):
        """Bascule l'état d'activation d'un Tier."""
        if tier_name in self.active_tiers:
            self.active_tiers.remove(tier_name)
        else:
            self.active_tiers.append(tier_name)

        col_key = f"Occ. {tier_name}"
        if col_key not in self.col_directions:
            self.col_directions[col_key] = True

        self._build_tier_menu()
        self._update_table()

    def _on_search(self, e):
        """Déclenché à chaque lettre tapée dans la barre de recherche."""
        self.search_query = e.control.value.lower()
        self._update_table()

    def _handle_sort(self, col_name: str):
        """Gère le basculement d'ordre de tri et la désignation de la colonne active."""
        if col_name not in self.col_directions:
            self.col_directions[col_name] = True

        if self.sort_col == col_name:
            self.col_directions[col_name] = not self.col_directions[col_name]
        else:
            self.sort_col = col_name

        self._update_table()

    def _update_table(self):
        """Filtre, trie et regénère entièrement le tableau d'affichage."""

        # 1. Définition des colonnes et des symboles de tri
        cols_definition = ["Tokens", "Occ. Tot."] + [f"Occ. {t}" for t in self.active_tiers]
        columns = []

        for col in cols_definition:
            is_asc = self.col_directions.get(col, True)

            # Rempli si colonne active, vide sinon
            if self.sort_col == col:
                sym = "▲" if is_asc else "▼"
            else:
                sym = "△" if is_asc else "▽"

            columns.append(
                ft.DataColumn(
                    label=ft.Text(f"{col} {sym}", weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    on_sort=lambda e, c=col: self._handle_sort(c)
                )
            )
        self._table.columns = columns

        # 2. Filtrage des mots
        filtered_words = [w for w in self.global_counts.keys() if self.search_query in w]

        # 3. Tri
        active_asc = self.col_directions.get(self.sort_col, True)

        def sort_key(word):
            if self.sort_col == "Tokens":
                return word
            elif self.sort_col == "Occ. Tot.":
                return self.global_counts[word]
            else:
                tier_target = self.sort_col.replace("Occ. ", "", 1)
                return self.tiers_data.get(tier_target, {}).get(word, 0)

        filtered_words.sort(key=sort_key, reverse=not active_asc)

        # 4. Génération des lignes
        rows = []
        for w in filtered_words:
            cells = [
                ft.DataCell(ft.Text(w, color=C["main_text"])),
                ft.DataCell(ft.Text(str(self.global_counts[w]), color=C["main_text"]))
            ]
            for tier in self.active_tiers:
                count = self.tiers_data[tier].get(w, 0)
                cells.append(ft.DataCell(ft.Text(str(count), color=C["main_text"])))

            rows.append(ft.DataRow(cells=cells))

        self._table.rows = rows

        if self.page:
            try:
                self.page.update()
            except Exception:
                pass

    # ── Moteur d'Export ──────────────────────────────────────────────────────

    def _export_excel(self, _e):
        """Exporte l'état actuel du tableau (filtré et trié) directement dans le fichier Excel configuré."""
        if not self._table.rows:
            self._status_text.value = "Aucune donnée à exporter."
            self._status_text.color = C["red"]
            self.page.update()
            return

        try:
            from backend.excel_utils import get_or_create_empty_sheet
        except ImportError:
            self._status_text.value = "Le module 'backend.excel_utils' est introuvable."
            self._status_text.color = C["red"]
            self.page.update()
            return

        # Récupération du chemin Excel configuré dans les paramètres
        data = self.sm.get_current_data() if self.sm else {}
        excel_path = data.get("excel_path", "").strip()

        # Si un chemin est défini dans la config, on l'utilise directement ; sinon fallback local
        export_path = excel_path if excel_path else "tokens_export.xlsx"

        self._status_text.value = "Écriture dans Excel..."
        self._status_text.color = C["validation_btn_default"]
        self.page.update()

        try:
            # get_or_create_empty_sheet va ouvrir excel_path (s'il existe),
            # chercher la première feuille vide (ou en créer une nouvelle), puis la retourner.
            wb, ws = get_or_create_empty_sheet(export_path)
            ws.title = "Tokenisation"

            # 1. Extraction et nettoyage des en-têtes (retrait des symboles de tri)
            headers = []
            for col in self._table.columns:
                raw_label = col.label.value
                clean_label = re.sub(r'\s*[▲▼△▽]', '', raw_label).strip()
                headers.append(clean_label)
            ws.append(headers)

            # 2. Écriture des données de l'instant T
            for row in self._table.rows:
                row_data = [cell.content.value for cell in row.cells]

                processed_row = []
                for val in row_data:
                    try:
                        processed_row.append(int(val))
                    except ValueError:
                        processed_row.append(val)

                ws.append(processed_row)

            # 3. Sauvegarde directement dans le fichier source
            wb.save(export_path)

            self._status_text.value = f"Données écrites avec succès dans :\n{export_path}"
            self._status_text.color = C["export_btn_default"]

        except Exception as e:
            self._status_text.value = f"Erreur lors de l'écriture Excel : {str(e)}"
            self._status_text.color = C["red"]

        self.page.update()

    # ── Cycle de Vie ─────────────────────────────────────────────────────────

    def on_show(self) -> None:
        """Appelé lorsque l'onglet est affiché à l'écran."""
        if not self.global_counts:
            self._load_data()

    def on_hide(self) -> None:
        pass