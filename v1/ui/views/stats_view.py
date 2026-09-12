"""
stats_view.py — Vue « Statistiques » : Statistiques linguistiques et contextuelles par Tiers.
"""

import os
import re
from collections import Counter
import flet as ft
from ui.theme import C

class StatsView:
    """Vue permettant l'analyse contextuelle d'un mot pivot selon une distance donnée."""

    def __init__(self, page: ft.Page, settings_manager=None) -> None:
        self.page = page
        self.sm = settings_manager

        # ── État des données ──────────────────────────────────────────────────
        self.tiers_raw_tokens: dict[str, list[str]] = {}
        self.active_tiers: list[str] = []

        # Résultats d'analyse
        self.total_pivot_global = 0
        self.total_pivot_tier = {}
        self.counts_global = Counter()
        self.counts_tier = {}

        self.direction = "Suivant"  # "Précédent" ou "Suivant"

        # ── Composants UI ─────────────────────────────────────────────────────
        self._in_pivot = ft.TextField(
            label="Pivot",
            hint_text="ex: du coup",
            bgcolor=C["white"],
            border_color=C["bg_container_border"],
            focused_border_color=C["validation_btn_default"],
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
            expand=True,
            height=45,
            content_padding=ft.Padding.symmetric(horizontal=10, vertical=0),
        )

        self._in_distance = ft.TextField(
            label="Écart(s)",
            value="1",
            hint_text="1, 2, 3...",
            bgcolor=C["white"],
            border_color=C["bg_container_border"],
            focused_border_color=C["validation_btn_default"],
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
            width=120,
            height=45,
            content_padding=ft.Padding.symmetric(horizontal=10, vertical=0),
        )

        # Boutons d'altération Précédent / Suivant
        self._btn_prev = ft.Button(
            "Précédent",
            on_click=lambda e: self._set_direction("Précédent"),
            style=self._get_dir_btn_style("Précédent"),
            height=45,
        )

        self._btn_next = ft.Button(
            "Suivant",
            on_click=lambda e: self._set_direction("Suivant"),
            style=self._get_dir_btn_style("Suivant"),
            height=45,
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
            tooltip="Sélectionner les Tiers à analyser",
        )

        self._btn_analyze = ft.Button(
            "Analyser",
            icon=None,
            on_click=self._run_analysis,
            style=ft.ButtonStyle(
                bgcolor={
                    ft.ControlState.DEFAULT: C["validation_btn_default"],
                    ft.ControlState.HOVERED: C["validation_btn_hovered"],
                },
                color=C["white"],
                shape=ft.RoundedRectangleBorder(radius=8),
            ),
            height=45,
        )

        self._table = ft.DataTable(
            columns=[
                ft.DataColumn(label=ft.Text("Type", weight=ft.FontWeight.BOLD, color=C["main_text"])),
                ft.DataColumn(label=ft.Text("Occ. Tot.", weight=ft.FontWeight.BOLD, color=C["main_text"])),
            ],
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
                    ft.Text("Statistiques", size=26, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Divider(height=1, color=C["divider_color"]),

                    # Barre de configuration
                    ft.Container(
                        bgcolor=C["bg_container"],
                        padding=12,
                        border_radius=12,
                        border=ft.Border.all(1, color=C["bg_container_border"]),
                        content=ft.Row(
                            controls=[
                                self._in_pivot,
                                self._btn_prev,
                                self._btn_next,
                                self._in_distance,
                                self._tiers_menu,
                                self._btn_analyze,
                            ],
                            spacing=12,
                            alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
                            vertical_alignment=ft.CrossAxisAlignment.CENTER,
                        )
                    ),

                    # Tableau de données
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

                    # Footer
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

    # ── Logique UI (Boutons Alternatifs) ─────────────────────────────────────

    def _get_dir_btn_style(self, btn_direction: str):
        is_active = (self.direction == btn_direction)
        return ft.ButtonStyle(
            bgcolor={
                ft.ControlState.DEFAULT: C["validation_btn_default"] if is_active else C["white"],
                ft.ControlState.HOVERED: C["validation_btn_hovered"] if is_active else "#F0F0F0",
            },
            color=C["white"] if is_active else C["main_text"],
            shape=ft.RoundedRectangleBorder(radius=8),
            side=ft.BorderSide(1, C["validation_btn_default"] if is_active else C["bg_container_border"]),
        )

    def _set_direction(self, new_dir: str):
        self.direction = new_dir
        self._btn_prev.style = self._get_dir_btn_style("Précédent")
        self._btn_next.style = self._get_dir_btn_style("Suivant")
        self.page.update()

    # ── Moteur d'Analyse et de Chargement ────────────────────────────────────

    def _read_text_auto(self, filepath: str) -> str:
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

    def _load_base_data(self) -> None:
        """Charge et tokenise les TextGrid séquentiellement pour garder l'ordre des mots."""
        data = self.sm.get_current_data() if self.sm else {}
        tg_paths = data.get("textgrid_path_files", "")

        # Réinitialisation — UNE SEULE FOIS, avant de traiter tous les
        # fichiers (sinon chaque nouveau fichier efface les précédents).
        self.tiers_raw_tokens.clear()
        self.active_tiers.clear()

        if not tg_paths:
            self._status_text.value = "Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres."
            self._status_text.color = C["red"]
            return

        tier_idx_pat = re.compile(r"item\s*\[\s*(\d+)\s*\]\s*:")
        name_pat = re.compile(r'name\s*=\s*"([^"]+)"')
        text_pat = re.compile(r'text\s*=\s*"([^"]*)"')

        missing_files = []
        error_files = []
        processed_count = 0

        for element in tg_paths:
            if not element or not os.path.exists(element):
                # On ne bloque plus tout le lot pour un seul fichier manquant.
                missing_files.append(element or "(chemin vide)")
                continue

            try:
                content = self._read_text_auto(element)
                lines = content.splitlines()

                current_tier = None
                # Tiers déjà vus DANS CE FICHIER : permet de distinguer un nom
                # de Tier qui réapparaît dans le même fichier (continuité
                # normale) d'un nom qui existait déjà via un fichier
                # précédent (auquel cas il faut poser une borne).
                tiers_seen_this_file = set()

                for line in lines:
                    stripped = line.strip()
                    if tier_idx_pat.search(stripped):
                        current_tier = "Unknown"
                        continue

                    if current_tier is not None:
                        n_match = name_pat.search(stripped)
                        if n_match and current_tier == "Unknown":
                            current_tier = n_match.group(1)
                            if current_tier not in self.tiers_raw_tokens:
                                self.tiers_raw_tokens[current_tier] = []
                            elif current_tier not in tiers_seen_this_file:
                                # Ce Tier existe déjà, mais via un AUTRE
                                # fichier : on insère une borne (None) pour
                                # empêcher qu'une recherche de contexte par
                                # distance ne chevauche deux fichiers
                                # différents (ex: fin d'interview 1 / début
                                # d'interview 2 de la même personne).
                                self.tiers_raw_tokens[current_tier].append(None)
                            tiers_seen_this_file.add(current_tier)
                            continue

                        t_match = text_pat.search(stripped)
                        if t_match and current_tier != "Unknown":
                            txt = t_match.group(1)
                            if txt.strip():
                                tokens = [w.lower() for w in re.split(r"[ ']+", txt) if w]
                                self.tiers_raw_tokens[current_tier].extend(tokens)

                processed_count += 1

            except Exception as ex:
                error_files.append(f"{os.path.basename(element)} ({ex})")

        # Activation par défaut de toutes les Tiers — UNE SEULE FOIS, une
        # fois tous les fichiers traités.
        self.active_tiers = list(self.tiers_raw_tokens.keys())
        self._build_tier_menu()

        if missing_files or error_files:
            parts = [f"{processed_count} fichier(s) chargé(s)"]
            if missing_files:
                parts.append(f"{len(missing_files)} introuvable(s)")
            if error_files:
                parts.append(f"{len(error_files)} en erreur")
            self._status_text.value = " · ".join(parts) + "."
            self._status_text.color = C["red"]
        else:
            self._status_text.value = ""

    def _build_tier_menu(self):
        self._tiers_menu.items.clear()
        for tier_name in self.tiers_raw_tokens.keys():
            is_checked = tier_name in self.active_tiers
            item = ft.PopupMenuItem(
                content=ft.Text(tier_name, color=C["main_text"]),
                checked=is_checked,
                on_click=lambda e, t=tier_name: self._toggle_tier(t)
            )
            self._tiers_menu.items.append(item)

    def _toggle_tier(self, tier_name: str):
        if tier_name in self.active_tiers:
            self.active_tiers.remove(tier_name)
        else:
            self.active_tiers.append(tier_name)
        self._build_tier_menu()
        self._run_analysis() # Relance l'analyse avec les nouvelles Tiers sélectionnées

    def _run_analysis(self, _e=None):
        """Recherche le pivot, calcule les occurrences aux distances souhaitées et met à jour le tableau."""
        if not self.tiers_raw_tokens:
            self._load_base_data()
            if not self.tiers_raw_tokens:
                self.page.update()
                return

        pivot_raw = (self._in_pivot.value or "").strip().lower()
        pivot_tokens = [w for w in re.split(r"[ ']+", pivot_raw) if w]

        if not pivot_tokens:
            self._status_text.value = "Veuillez entrer un pivot valide."
            self._status_text.color = C["red"]
            self.page.update()
            return

        distances_str = (self._in_distance.value or "1").replace(" ", "").split(",")
        distances = []
        for d in distances_str:
            try:
                distances.append(int(d))
            except ValueError:
                pass

        if not distances:
            distances = [1]

        # Reset des compteurs
        self.total_pivot_global = 0
        self.total_pivot_tier = {t: 0 for t in self.active_tiers}
        self.counts_global.clear()
        self.counts_tier = {t: Counter() for t in self.active_tiers}

        p_len = len(pivot_tokens)
        pivot_display = " ".join(pivot_tokens)

        def _has_file_boundary(tokens: list, a: int, b: int) -> bool:
            """True si une borne de fichier (None) se trouve entre les index
            a et b inclus — empêche un contexte de chevaucher deux fichiers
            différents (ex: fin d'un fichier / début du suivant)."""
            lo, hi = (a, b) if a <= b else (b, a)
            lo = max(lo, 0)
            hi = min(hi, len(tokens) - 1)
            return any(tokens[k] is None for k in range(lo, hi + 1))

        # Moteur de recherche
        for tier in self.active_tiers:
            tokens = self.tiers_raw_tokens.get(tier, [])
            i = 0
            while i <= len(tokens) - p_len:
                window = tokens[i:i+p_len]
                if None in window:
                    # Le pivot chevaucherait lui-même une borne de fichier.
                    i += 1
                    continue

                if window == pivot_tokens:
                    self.total_pivot_tier[tier] += 1
                    self.total_pivot_global += 1

                    for d in distances:
                        if self.direction == "Suivant":
                            target_idx = i + p_len + d - 1
                            span = (i + p_len, target_idx)
                        else:
                            target_idx = i - d
                            span = (target_idx, i - 1)

                        # Vérification des bordures : hors limites, OU le
                        # trajet du pivot jusqu'au mot de contexte traverse
                        # une frontière entre deux fichiers différents.
                        if 0 <= target_idx < len(tokens) and not _has_file_boundary(tokens, *span):
                            target_word = tokens[target_idx]

                            # Formatage des espaces / underscores
                            if d == 1:
                                underscores = " "
                            else:
                                underscores = " " + " ".join(["_"] * (d - 1)) + " "

                            if self.direction == "Suivant":
                                display_str = f"{pivot_display}{underscores}{target_word}".replace("  ", " ")
                            else:
                                display_str = f"{target_word}{underscores}{pivot_display}".replace("  ", " ")

                            self.counts_tier[tier][display_str] += 1
                            self.counts_global[display_str] += 1
                i += 1

        self._update_table()
        self._status_text.value = "Analyse terminée avec succès."
        self._status_text.color = C["export_btn_default"]
        self.page.update()

    def _update_table(self):
        """Reconstruit les colonnes et lignes du tableau basé sur les résultats courants."""
        columns = [
            ft.DataColumn(label=ft.Text("Type", weight=ft.FontWeight.BOLD, color=C["main_text"])),
            ft.DataColumn(label=ft.Text("Occ. Tot.", weight=ft.FontWeight.BOLD, color=C["main_text"])),
        ]
        for tier in self.active_tiers:
            columns.append(ft.DataColumn(label=ft.Text(f"Occ. {tier}", weight=ft.FontWeight.BOLD, color=C["main_text"])))

        self._table.columns = columns

        # Tri descendant par occurrence globale
        sorted_items = self.counts_global.most_common()
        rows = []

        for display_str, glob_count in sorted_items:
            cells = [
                ft.DataCell(ft.Text(display_str, color=C["main_text"])),
                ft.DataCell(ft.Text(f"{glob_count} / {self.total_pivot_global}", color=C["main_text"])),
            ]

            for tier in self.active_tiers:
                tier_count = self.counts_tier.get(tier, {}).get(display_str, 0)
                tot_tier = self.total_pivot_tier.get(tier, 0)
                cells.append(ft.DataCell(ft.Text(f"{tier_count} / {tot_tier}", color=C["main_text"])))

            rows.append(ft.DataRow(cells=cells))

        self._table.rows = rows

    # ── Moteur d'Export ──────────────────────────────────────────────────────

    def _export_excel(self, _e):
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

        data = self.sm.get_current_data() if self.sm else {}
        excel_path = data.get("excel_path", "").strip()
        export_path = excel_path if excel_path else "statistiques_export.xlsx"

        try:
            wb, ws = get_or_create_empty_sheet(export_path)
            ws.title = "Statistiques"

            # En-têtes
            headers = [col.label.value for col in self._table.columns]
            ws.append(headers)

            # Données
            for row in self._table.rows:
                ws.append([cell.content.value for cell in row.cells])

            wb.save(export_path)

            self._status_text.value = f"Export réussi :\n{export_path}"
            self._status_text.color = C["export_btn_default"]

        except Exception as e:
            self._status_text.value = f"Erreur lors de l'écriture Excel : {str(e)}"
            self._status_text.color = C["red"]

        self.page.update()

    # ── Cycle de Vie ─────────────────────────────────────────────────────────

    def on_show(self) -> None:
        if not self.tiers_raw_tokens:
            self._load_base_data()
        self._update_table()
        if self.page:
            try:
                self.page.update()
            except Exception:
                pass

    def on_hide(self) -> None:
        pass