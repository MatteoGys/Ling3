import os
import re
import math
import flet as ft
from ui.theme import C
from backend.excel_utils import get_or_create_empty_sheet

try:
    from openpyxl import Workbook
except ImportError:
    Workbook = None


class ConcordancierView:
    """Vue permettant de chercher des occurrences (pivots) et d'exporter un concordancier Excel (.xlsx)."""

    def __init__(self, page: ft.Page, settings_manager=None) -> None:
        self.page = page
        self.sm = settings_manager

        # ── État des données ──────────────────────────────────────────────────
        self.default_columns = [
            "ID", "Tier Nom", "Tier Numéro", "Tier Lettre", "x_min", "x_max",
            "durée_occurence", "contexte gauche", "pivot", "contexte droit", "POS_PAUSES_VIDES"
        ]
        # Modification 1 : Uniquement ID, Contexte gauche, Pivot et Contexte Droit affichés au démarrage
        self.active_columns = ["ID", "contexte gauche", "pivot", "contexte droit"]

        # ── Composants UI ─────────────────────────────────────────────────────
        self._in_pivots = ft.TextField(
            label="Pivots à chercher",
            hint_text="Ex: euh\\hum\\du coup",
            expand=True,
            bgcolor=C["white"],
            border_color=C["bg_container_border"],
            focused_border_color=C["validation_btn_default"],
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
        )

        self._in_ctx_left = ft.TextField(
            label="Mots avant\n(Gauche)",
            value="5",
            width=150,
            bgcolor=C["white"],
            border_color=C["bg_container_border"],
            focused_border_color=C["validation_btn_default"],
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
            input_filter=ft.NumbersOnlyInputFilter(),
        )

        self._in_ctx_right = ft.TextField(
            label="Mots après\n(Droite)",
            value="5",
            width=150,
            bgcolor=C["white"],
            border_color=C["bg_container_border"],
            focused_border_color=C["validation_btn_default"],
            color=C["main_text"],
            label_style=ft.TextStyle(color=C["secondary_text"]),
            input_filter=ft.NumbersOnlyInputFilter(),
        )

        self._status_text = ft.Text("", size=14, color=C["secondary_text"])

        # Ligne horizontale pour les colonnes Excel avec défilement
        self._columns_row = ft.Row(scroll=ft.ScrollMode.ADAPTIVE, spacing=16, expand=True)

        # Structure racine
        self.root = ft.Container(
            expand=True,
            bgcolor=C["bg_onglet"],
            padding=28,
            content=ft.Column(
                expand=True,
                spacing=20,
                controls=[
                    ft.Text("Concordancier Excel", size=26, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                    ft.Divider(height=1, color=C["divider_color"]),

                    # Section Configuration (Pivots et Contextes)
                    ft.Container(
                        bgcolor=C["bg_container"],
                        padding=12,
                        border_radius=12,
                        border=ft.Border.all(1, color=C["bg_container_border"]),
                        content=ft.Row([self._in_pivots, self._in_ctx_left, self._in_ctx_right], spacing=16)
                    ),

                    # Section Paramètres Drag & Drop
                    ft.Row([
                        ft.Column([
                            ft.Text("Ordre des colonnes :",
                                    size=16, weight=ft.FontWeight.BOLD, color=C["main_text"]),
                            ft.Text("Faites glisser les colonnes pour réorganiser l'export. Ajoutez ou retirez des étiquettes.",
                                    size=13, color="#888888"),
                        ], spacing=2),
                        ft.Container(expand=True),
                        ft.TextButton(
                            content=ft.Text("Réinitialiser l'ordre", color=C["validation_btn_default"]),
                            icon=ft.Icons.RESTORE,
                            icon_color=C["validation_btn_default"],
                            style=ft.ButtonStyle(color=C["validation_btn_default"]),
                            on_click=self._reset_columns
                        )
                    ]),

                    # Zone encadrée contenant la ligne horizontale des paramètres
                    ft.Container(
                        height=380,
                        bgcolor=C["white"],
                        border_radius=12,
                        border=ft.Border.all(1, color=C["bg_container_border"]),
                        padding=16,
                        content=self._columns_row
                    ),

                    # Footer : Statut et Bouton de génération
                    ft.Row(
                        alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
                        controls=[
                            self._status_text,
                            ft.Button(
                                "Générer le fichier Excel",
                                on_click=self._generate_excel,
                                style=ft.ButtonStyle(
                                    bgcolor={ft.ControlState.DEFAULT: C["validation_btn_default"], ft.ControlState.HOVERED: C["validation_btn_hovered"]},
                                    color=C["white"],
                                    shape=ft.RoundedRectangleBorder(radius=8),
                                ),
                                height=45,
                            )
                        ]
                    )
                ]
            )
        )

        self._build_columns_ui()

    # ── Helpers de formatage ─────────────────────────────────────────────────

    def _smart_wrap(self, text: str, max_len: int = 12) -> str:
        """Coupe le texte de manière intelligente s'il est trop long."""
        # Modification 3 : Formatage spécifique et propre pour POS_PAUSES_VIDES
        if text == "POS_PAUSES_VIDES":
            return "POS_\nPAUSES_\nVIDES"

        if text == "durée_occurence":
            return "durée_\noccurence"

        if len(text) <= max_len:
            return text

        last_underscore = text.rfind('_')
        if last_underscore != -1:
            return text[:last_underscore + 1] + '\n' + text[last_underscore + 1:]

        return text

    # ── Logique UI (Drag & Drop, Ajout, Suppression) ─────────────────────────

    def _reset_columns(self, _e):
        self.active_columns = ["ID", "contexte gauche", "pivot", "contexte droit"]
        self._build_columns_ui()

    def _remove_col(self, e):
        col_to_remove = e.control.data
        if col_to_remove in self.active_columns:
            self.active_columns.remove(col_to_remove)
            self._build_columns_ui()

    def _add_col(self, e):
        col_to_add = e.control.data
        if col_to_add not in self.active_columns:
            self.active_columns.append(col_to_add)
            self._build_columns_ui()

    def _on_drag_accept(self, e: ft.DragTargetEvent):
        src_control = self.page.get_control(e.src_id)
        src_col = src_control.data
        dest_col = e.control.data

        if src_col != dest_col and src_col in self.active_columns and dest_col in self.active_columns:
            idx_src = self.active_columns.index(src_col)
            idx_dest = self.active_columns.index(dest_col)

            item = self.active_columns.pop(idx_src)
            self.active_columns.insert(idx_dest, item)

            self._build_columns_ui()

    def _build_columns_ui(self):
        """Construit l'interface des tuiles actives et des colonnes disponibles par blocs de 6."""
        self._columns_row.controls.clear()

        TILE_WIDTH = 70
        TILE_HEIGHT = 290

        # 1. Ajout des tuiles actives
        for col in self.active_columns:
            display_text = self._smart_wrap(col)

            item_ui = ft.Container(
                data=col,
                width=TILE_WIDTH,
                height=TILE_HEIGHT,
                bgcolor=C["bg_onglet"],
                border_radius=8,
                border=ft.Border.all(1, color=C["bg_container_border"]),
                padding=ft.Padding.symmetric(vertical=10),
                content=ft.Column(
                    alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
                    horizontal_alignment=ft.CrossAxisAlignment.CENTER,
                    controls=[
                        ft.Icon(ft.Icons.DRAG_INDICATOR, color=C["secondary_text"], size=20),
                        ft.Container(
                            expand=True,
                            alignment=ft.Alignment(0, 0),
                            content=ft.Text(
                                display_text,
                                color=C["main_text"],
                                weight=ft.FontWeight.BOLD,
                                size=13,
                                text_align=ft.TextAlign.CENTER,
                                rotate=math.pi / -2
                            )
                        ),
                        ft.IconButton(ft.Icons.CLOSE, icon_size=18, icon_color=C["red"], tooltip="Retirer", data=col,
                                      on_click=self._remove_col)
                    ]
                )
            )

            dragging_ui = ft.Container(
                width=TILE_WIDTH,
                height=TILE_HEIGHT,
                bgcolor=C["white"],
                border_radius=8,
                border=ft.Border.all(2, color=C["validation_btn_default"]),
                shadow=ft.BoxShadow(spread_radius=1, blur_radius=10, color="#44000000"),
                padding=ft.Padding.symmetric(vertical=10),
                content=ft.Column(
                    alignment=ft.MainAxisAlignment.SPACE_BETWEEN,
                    horizontal_alignment=ft.CrossAxisAlignment.CENTER,
                    controls=[
                        ft.Icon(ft.Icons.DRAG_INDICATOR, color=C["validation_btn_default"], size=20),
                        ft.Container(
                            expand=True,
                            alignment=ft.Alignment(0, 0),
                            content=ft.Text(
                                display_text,
                                color=C["validation_btn_default"],
                                weight=ft.FontWeight.BOLD,
                                size=13,
                                text_align=ft.TextAlign.CENTER,
                                rotate=math.pi / -2
                            )
                        ),
                        ft.Icon(ft.Icons.CLOSE, size=18, color="#FFFFFF00")
                    ]
                )
            )

            draggable = ft.Draggable(
                group="cols",
                data=col,
                content=item_ui,
                content_when_dragging=dragging_ui,
            )

            drag_target = ft.DragTarget(
                group="cols",
                data=col,
                on_accept=self._on_drag_accept,
                content=draggable
            )

            self._columns_row.controls.append(drag_target)

        # 2. Zone des colonnes disponibles découpées par colonnes de 6 éléments maximum
        available_cols = [c for c in self.default_columns if c not in self.active_columns]

        if available_cols:
            # Ajout d'un séparateur vertical si des colonnes actives sont présentes
            if self.active_columns:
                self._columns_row.controls.append(
                    ft.VerticalDivider(width=25, color=C["bg_container_border"]),
                )

            chunk_size = 6
            chunks = [available_cols[i:i + chunk_size] for i in range(0, len(available_cols), chunk_size)]

            for idx, chunk in enumerate(chunks):
                available_buttons = [
                    ft.TextButton(
                        content=ft.Text(col, color=C["main_text"]),
                        icon=ft.Icons.ADD_CIRCLE_OUTLINE,
                        icon_color=C["validation_btn_default"],
                        style=ft.ButtonStyle(color=C["main_text"]),
                        data=col,
                        on_click=self._add_col
                    )
                    for col in chunk
                ]

                available_container = ft.Container(
                    width=220,
                    alignment=ft.Alignment(-1, -1),
                    padding=ft.Padding.only(top=10, left=5, right=5),
                    content=ft.Column(
                        controls=[
                            ft.Text("Colonnes disponibles :" if idx == 0 else "", size=14, color=C["secondary_text"],
                                    weight=ft.FontWeight.BOLD),
                            ft.Text("Cliquez pour ajouter." if idx == 0 else "", size=12, color=C["secondary_text"]),
                            ft.Container(height=5 if idx == 0 else 32),  # Aligne les boutons avec les autres colonnes
                            ft.Column(controls=available_buttons, spacing=6,
                                      horizontal_alignment=ft.CrossAxisAlignment.START)
                        ],
                        spacing=2,
                        alignment=ft.MainAxisAlignment.START,
                        horizontal_alignment=ft.CrossAxisAlignment.START
                    )
                )
                self._columns_row.controls.append(available_container)

        self.page.update()

    # ── Moteur d'Extraction ──────────────────────────────────────────────────

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

    def _generate_excel(self, _e):
        # Vérification préalable de la présence d'openpyxl
        if Workbook is None:
            self._status_text.value = "Le module 'openpyxl' est requis."
            self._status_text.color = C["red"]
            self.page.update()
            return

        data = self.sm.get_current_data() if self.sm else {}
        tg_paths = data.get("textgrid_path_files", "")
        separator = data.get("separator", "\\")
        excel_path = data.get("excel_path", "").strip()
        export_path = excel_path if excel_path else "tokens_export.xlsx"

        if not tg_paths:
            self._status_text.value = "Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres."
            self._status_text.color = C["red"]
            self.page.update()
            return

        # ── Validations — UNE SEULE FOIS, avant la boucle : elles ne dépendent
        # pas du fichier en cours. ─────────────────────────────────────────
        pivots_raw = self._in_pivots.value or ""
        pivots = [p.strip() for p in pivots_raw.split(separator) if p.strip()]

        if not pivots:
            self._status_text.value = "Veuillez indiquer au moins un pivot."
            self._status_text.color = C["red"]
            self.page.update()
            return

        try:
            nb_left = int(self._in_ctx_left.value)
            nb_right = int(self._in_ctx_right.value)
        except ValueError:
            self._status_text.value = "Les contextes doivent être des nombres entiers."
            self._status_text.color = C["red"]
            self.page.update()
            return

        self._status_text.value = "Analyse en cours..."
        self._status_text.color = C["validation_btn_default"]
        self.page.update()

        # Précompilation des Regex — une seule fois pour tous les fichiers
        compiled_pivots = [re.compile(r'\b' + re.escape(pivot) + r'\b', re.IGNORECASE) for pivot in pivots]
        name_pat = re.compile(r'name\s*=\s*"([^"]+)"')
        xmin_pat = re.compile(r'xmin\s*=\s*([0-9.]+)')
        xmax_pat = re.compile(r'xmax\s*=\s*([0-9.]+)')
        text_pat = re.compile(r'text\s*=\s*"([^"]*)"')

        # ── Résultats cumulés sur TOUS les fichiers, avant l'écriture Excel ──
        results = []
        occurrence_id = 1
        tier_counter = 0
        missing_files = []
        error_files = []
        processed_count = 0

        for element in tg_paths:
            if not element or not os.path.exists(element):
                # On ne bloque plus tout le lot pour un fichier manquant.
                missing_files.append(element or "(chemin vide)")
                continue

            try:
                content = self._read_text_auto(element)

                # Parsing TextGrid
                tiers = []
                items = re.split(r"item\s*\[\s*\d+\s*\]\s*:", content)[1:]

                for item_str in items:
                    name_m = name_pat.search(item_str)
                    if not name_m: continue

                    tier_counter += 1
                    tier = {'index': tier_counter, 'name': name_m.group(1), 'intervals': []}
                    interval_blocks = re.split(r"intervals\s*\[\s*\d+\s*\]\s*:", item_str)[1:]

                    for block in interval_blocks:
                        xmin_m = xmin_pat.search(block)
                        xmax_m = xmax_pat.search(block)
                        text_m = text_pat.search(block)

                        if xmin_m and xmax_m and text_m:
                            t_val = text_m.group(1)
                            tier['intervals'].append({
                                'xmin': float(xmin_m.group(1)),
                                'xmax': float(xmax_m.group(1)),
                                'text': t_val,
                                'is_empty': not t_val.strip()
                            })
                    tiers.append(tier)

                # Extraction (cumulée dans `results`, commun à tous les fichiers)
                for tier in tiers:
                    flat_words = []
                    interval_word_start = []
                    for t in tier['intervals']:
                        interval_word_start.append(len(flat_words))
                        flat_words.extend(t['text'].split())
                    interval_word_start.append(len(flat_words))  # sentinelle finale

                    for idx, interval in enumerate(tier['intervals']):
                        txt = interval['text']
                        if interval['is_empty']: continue

                        for pattern in compiled_pivots:
                            for match in pattern.finditer(txt):
                                ip_empty = True if idx == 0 else tier['intervals'][idx - 1]['is_empty']
                                is_empty = True if idx == len(tier['intervals']) - 1 else tier['intervals'][idx + 1][
                                    'is_empty']

                                if ip_empty and is_empty:
                                    pos = "Isolé"
                                elif ip_empty and not is_empty:
                                    pos = "Initial"
                                elif not ip_empty and is_empty:
                                    pos = "Final"
                                else:
                                    pos = "Médian"

                                texte_inter_gauche = txt[:match.start()]
                                texte_inter_droit = txt[match.end():]

                                mots_gauche_locaux = texte_inter_gauche.split()
                                mots_droit_locaux = texte_inter_droit.split()

                                if nb_left > 0:
                                    if len(mots_gauche_locaux) >= nb_left:
                                        mots_gauche = mots_gauche_locaux[-nb_left:]
                                    else:
                                        manquants = nb_left - len(mots_gauche_locaux)
                                        debut_global = interval_word_start[idx]
                                        prefixe = flat_words[max(0, debut_global - manquants):debut_global]
                                        mots_gauche = prefixe + mots_gauche_locaux
                                else:
                                    mots_gauche = []

                                if nb_right > 0:
                                    if len(mots_droit_locaux) >= nb_right:
                                        mots_droit = mots_droit_locaux[:nb_right]
                                    else:
                                        manquants = nb_right - len(mots_droit_locaux)
                                        fin_global = interval_word_start[idx + 1]
                                        suffixe = flat_words[fin_global:fin_global + manquants]
                                        mots_droit = mots_droit_locaux + suffixe
                                else:
                                    mots_droit = []

                                ctx_gauche = " ".join(mots_gauche)
                                ctx_droit = " ".join(mots_droit)

                                row_data = {
                                    "ID": occurrence_id,
                                    "Tier Nom": tier['name'],
                                    "Tier Numéro": tier['index'],
                                    "Tier Lettre": chr(64 + tier['index']) if tier['index'] <= 26 else str(
                                        tier['index']),
                                    "x_min": round(interval['xmin'], 4),
                                    "x_max": round(interval['xmax'], 4),
                                    "durée_occurence": round(interval['xmax'] - interval['xmin'], 4),
                                    "contexte gauche": ctx_gauche,
                                    "pivot": match.group(0),
                                    "contexte droit": ctx_droit,
                                    "POS_PAUSES_VIDES": pos
                                }

                                final_row = [row_data[col] for col in self.active_columns]
                                results.append(final_row)
                                occurrence_id += 1

                processed_count += 1

            except Exception as ex:
                error_files.append(f"{os.path.basename(element)} ({ex})")

        # ── Écriture Excel — UNE SEULE FOIS, après avoir traité tous les
        # fichiers : même feuille, en-têtes écrits une fois, toutes les lignes
        # à la suite. ─────────────────────────────────────────────────────
        try:
            wb, ws = get_or_create_empty_sheet(export_path)
            ws.title = "Concordancier"
            ws.append(self.active_columns)
            for row in results:
                ws.append(row)
            wb.save(export_path)
        except Exception as e:
            self._status_text.value = f"Erreur lors de l'écriture Excel : {str(e)}"
            self._status_text.color = C["red"]
            self.page.update()
            return

        parts = [f"{processed_count} fichier(s) analysé(s)", f"{len(results)} occurrence(s)"]
        if missing_files:
            parts.append(f"{len(missing_files)} introuvable(s)")
        if error_files:
            parts.append(f"{len(error_files)} en erreur")

        if missing_files or error_files:
            self._status_text.value = " · ".join(parts) + f"\nExporté dans : {export_path}"
            self._status_text.color = C["red"]
        else:
            self._status_text.value = f"Export Excel réussi dans :\n{export_path} ({len(results)} occurrences, {processed_count} fichier(s))"
            self._status_text.color = "#388e3c"

        self.page.update()

    # ── Cycle de vie ─────────────────────────────────────────────────────────

    def on_show(self) -> None:
        pass

    def on_hide(self) -> None:
        pass