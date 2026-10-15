<div align="center">
  <h1>Ling3</h1>
  <p><b>Téléchargez ici / Download here</b></p>

  <p>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_x64-setup.exe"><img alt="Windows x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20(x64)-0078D4"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_amd64.AppImage"><img alt="Linux x64 / ARM64" src="https://img.shields.io/badge/Linux-x64%20%2F%20ARM64-F4BC00"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_aarch64.dmg"><img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon%20(M1%2B)-007AFF"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_x64.dmg"><img alt="macOS Intel" src="https://img.shields.io/badge/macOS-Intel%20(x86__64)-555555"></a>
    <br>
    <br>
    <a href="https://github.com/MatteoGys/Ling3/releases"><img alt="GitHub Release" src="https://img.shields.io/github/v/release/MatteoGys/Ling3?include_prereleases&color=2EA44F&label=version"></a>
    <a href="https://github.com/MatteoGys/Ling3/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-8957E5"></a>
  </p>
</div>

<br>
<br>
[English below]

## A propos
<p align="justify">
  
_Ling3_ est un outil d'analyse de fichiers TextGrid (Praat) pensé pour le travail linguistique. L'application permet la détection de chevauchements, la préparation et la conversion de fichiers, le comptage de tokens, la génération d'un concordancier sur mesure ainsi que l'étude du contexte d'un pivot.
</p>

## Fonctionnalités des modules

### 1. Paramètres
<p align="justify">
  
Avant d'utiliser les outils d'analyse, l'onglet _Paramètres_ permet de configurer :
- **Fichiers TextGrid** : Le ou les fichiers a traiter (.TextGrid ou .txt), requis pour tous les onglets d'analyse.
- **Fichier Excel** : Le classeur (.xlsx) destiné à recevoir les tableaux exportés (Tokenisation, Concordancier, Cooccurrences).
- **Séparateur** : Caractère utilisé pour distinguer plusieurs pivots saisis dans un même champ (par défaut, un backslash `\`).
</p>

### 2. Chevauchements
<p align="justify">
  
Vérifie qu'aucun intervalle annoté ne chevauche un intervalle d'une autre tier au sein d'un même fichier TextGrid. En cas de conflit, l'application liste la tier source, le numéro de l'intervalle, son timecode de fin, son texte et les tiers en cause pour faciliter la localisation.
</p>

### 3. Modifications
<p align="justify">
  
Regroupe les opérations de préparation des fichiers :
- **Conversion TRS vers TextGrid** : Transforme des fichiers Transcriber (.trs, format CFPP) en fichiers TextGrid enregistrés sur le Bureau (basé sur l'algorithme _LingTools_ de l'Univérsité de l'Oregon auquel a été ajouté un correcteur d'erreurs).
- **Remplacement de texte et fusion des silences** : Permet de modifier une chaine de caractères sur l'ensemble d'un fichier et de fusionner les intervalles de silence consécutifs.
- **Extraction par mots pivots** : Isole dans un nouveau fichier les intervalles contenant un pivot spécifique, tout en fusionnant les intervalles adjacents en silence.
</p>

### 4. Tokenisation
<p align="justify">
  
Comptabilise chaque token (mot) présent dans les fichiers TextGrid et restitue les occurrences sous forme de tableau. Offre des fonctionnalités de recherche, de tri par colonne et d'export direct vers Excel.
</p>

### 5. Concordancier
<p align="justify">
  
Génère un concordancier sur mesure dans un fichier Excel en extrayant chaque occurrence d'un pivot entourée de son contexte gauche et droit. Il est possible de configurer l'ordre des colonnes, le nombre de mots de contexte, ainsi que d'ajouter des métadonnées avancées (timecodes, durée, position par rapport aux silences).
</p>

### 6. Cooccurrences
<p align="justify">
  
Etudie l'environnement immédiat d'un pivot en dénombrant les mots présents à une distance définie. Le résultat présente le type de contexte associe et le nombre d'occurrences globales ou par tier, exportable sous format Excel.
</p>

### Licence
<p align="justify">
  
Ce projet est distribue sous les termes de la licence mentionnée dans le fichier LICENSE.
</p>

_L'architecture de l'application se trouve en dessous._

<br>
<br>
<br>
[English here]

## About
<p align="justify">
  
_Ling3_ is a TextGrid (Praat) file analysis tool designed for linguistic research. The application enables overlap detection, file preparation and conversion, token counting, custom concordance generation, as well as the study of a pivot word's statistical context.
</p>

## Module Features

### 1. Settings
<p align="justify">
  
Before using the analysis tools, the <i>Settings</i> tab allows you to configure:
- **TextGrid Files**: The file(s) to process (.TextGrid or .txt), required for all analysis tabs.
- **Excel File**: The workbook (.xlsx) designed to receive exported tables (Tokenization, Concordancer, Co-occurrences).
- **Separator**: Character used to distinguish multiple pivots entered in the same field (default is a backslash `\`).
</p>


### 2. Overlaps
<p align="justify">
  
Verifies that no annotated interval overlaps with an interval from another tier within the same TextGrid file. In case of conflict, the application lists the source tier, interval number, end timecode, text, and conflicting tiers to facilitate localization.
</p>

### 3. Modifications
<p align="justify">
  
Groups file preparation operations together:
- **TRS to TextGrid Conversion**: Converts Transcriber files (.trs, CFPP format) into TextGrid files saved to the Desktop (based on the <i>LingTools</i> algorithm from the University of Oregon, enhanced with an error corrector).
- **Text Replacement and Silence Merging**: Allows replacing a character string across an entire file and merging consecutive silence intervals.
- **Extraction by Pivot Words**: Isolates intervals containing a specific pivot into a new file, while merging adjacent intervals into silence.
</p>

### 4. Tokenization
<p align="justify">
  
Counts each token (word) present in the TextGrid files and presents occurrences in table format. Offers search features, column sorting, and direct export to Excel.
</p>

### 5. Concordancer
<p align="justify">
  
Generates a custom concordancer in an Excel file by extracting each occurrence of a pivot word surrounded by its left and right context. You can configure column order, context word count, and add advanced metadata (timecodes, duration, position relative to silences).
</p>

### 6. Co-occurrences
<p align="justify">
  
Studies the immediate environment of a pivot word by counting words present within a defined distance. Results show the associated context type and occurrence counts globally or per tier, exportable to Excel format.
</p>

### License
<p align="justify">

This project is distributed under the terms of the license specified in the LICENSE file.
</p>

_The application architecture can be found below._

---

### Architecture
![Architecture de Ling3](assets/diagram.png)

```text
Directory structure:
└── v2/
    ├── Cargo.toml
    ├── compile.sh
    ├── index.html
    ├── styles.css
    ├── todo.txt
    ├── Trunk.toml
    ├── src/
    │   ├── app.rs
    │   └── main.rs
    ├── src-tauri/
    │   ├── build.rs
    │   ├── Cargo.toml
    │   ├── tauri.conf.json
    │   ├── capabilities/
    │   │   └── default.json
    │   ├── gen/
    │   │   └── schemas/
    │   │       └── capabilities.json
    │   ├── icons/
    │   │   └── android/
    │   │       ├── mipmap-anydpi-v26/
    │   │       │   └── ic_launcher.xml
    │   │       └── values/
    │   │           └── ic_launcher_background.xml
    │   └── src/
    │       ├── commands.rs
    │       ├── config.rs
    │       ├── excel.rs
    │       ├── lib.rs
    │       ├── main.rs
    │       ├── models.rs
    │       ├── settings.rs
    │       └── textgrid/
    │           ├── concordance.rs
    │           ├── cooccurrences.rs
    │           ├── editor.rs
    │           ├── excel.rs
    │           ├── mod.rs
    │           ├── overlap.rs
    │           ├── parser.rs
    │           ├── tg_check.rs
    │           ├── tokenisation.rs
    │           └── trs_convert.rs
    └── ui/
        ├── Cargo.toml
        ├── index.html
        ├── Trunk.toml
        ├── src/
        │   ├── api.rs
        │   ├── app.rs
        │   ├── decoration.rs
        │   ├── icons.rs
        │   ├── main.rs
        │   ├── theme.rs
        │   └── views/
        │       ├── chevauchement.rs
        │       ├── concordancier.rs
        │       ├── cooccurrences.rs
        │       ├── info.rs
        │       ├── mod.rs
        │       ├── modification_textgrid.rs
        │       ├── placeholder.rs
        │       ├── settings.rs
        │       └── tokenisation.rs
        └── styles/
            └── theme.css
