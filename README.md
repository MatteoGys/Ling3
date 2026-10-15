<div align="center">
  <img width="64" height="64" alt="Ling3 Logo" src="https://raw.githubusercontent.com/MatteoGys/Ling3/v2/src-tauri/icons/icon.png" />
  <h1>Ling3</h1>
  <p><b>Application de bureau dédiée à l'analyse de fichiers TextGrid (Praat) pour la recherche linguistique</b></p>

  <p>
    <a href="https://github.com/MatteoGys/Ling3/releases"><img alt="GitHub Release" src="https://img.shields.io/github/v/release/MatteoGys/Ling3?include_prereleases&color=0A84FF&label=version"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_x64-setup.exe"><img alt="Windows x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20(x64)-0078D4"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_amd64.AppImage"><img alt="Linux x64 / ARM64" src="https://img.shields.io/badge/Linux-x64%20%2F%20ARM64-FCC624"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_aarch64.dmg"><img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon%20(M1%2B)-0A84FF"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases/download/v2.0.0/Ling3_2.0.0_x64.dmg"><img alt="macOS Intel" src="https://img.shields.io/badge/macOS-Intel%20(x86__64)-555555"></a>
    <a href="https://github.com/MatteoGys/Ling3/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green"></a>
  </p>
</div>

---

## A propos
<p align="justify">
  
_Ling3_ est un outil d'analyse de fichiers TextGrid (Praat) pensé pour le travail linguistique. L'application permet la détection de chevauchements, la préparation et la conversion de fichiers, le comptage de tokens, la génération d'un concordancier sur mesure ainsi que l'étude du contexte d'un pivot.
</p>
---

## Fonctionnalités des modules

### 1. Paramètres
<p align="justify">
Avant d'utiliser les outils d'analyse, l'onglet _Paramètres_ permet de configurer :
- **Fichiers TextGrid** : Le ou les fichiers a traiter (.TextGrid ou .txt), requis pour tous les onglets d'analyse.
- **Fichier Excel** : Le classeur (.xlsx) destiné à recevoir les tableaux exportés (Tokenisation, Concordancier, Cooccurrences).
- **Separateur** : Caractère utilisé pour distinguer plusieurs pivots saisis dans un même champ (par défaut, un backslash `\`).
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

### Architecture du projet
[![Architecture diagram of matteogys/ling3](https://gitdiagram.com/matteogys/ling3/diagram.png)](https://gitdiagram.com/matteogys/ling3?utm_source=readme&utm_medium=picture)
### Licence
<p align="justify">
Ce projet est distribue sous les termes de la licence mentionnée dans le fichier LICENSE.
</p>

