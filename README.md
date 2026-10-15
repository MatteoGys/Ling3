<div align="center">
  <img width="64" height="64" alt="Ling3 Logo" src="https://raw.githubusercontent.com/MatteoGys/Ling3/v2/src-tauri/icons/icon.png" />
  <h1>Ling3</h1>
  <p><b>Application de bureau dediee a l'analyse de fichiers TextGrid (Praat) pour la recherche linguistique</b></p>

  <p>
    <a href="https://github.com/MatteoGys/Ling3/releases"><img alt="GitHub Release" src="https://img.shields.io/github/v/release/MatteoGys/Ling3?include_prereleases&color=0A84FF&label=version"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases"><img alt="Windows x64" src="https://img.shields.io/badge/Windows-10%20%2F%2011%20(x64)-0078D4"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases"><img alt="Linux x64 / ARM64" src="https://img.shields.io/badge/Linux-x64%20%2F%20ARM64-FCC624"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases"><img alt="macOS Apple Silicon" src="https://img.shields.io/badge/macOS-Apple%20Silicon%20(M1%2B)-0A84FF"></a>
    <a href="https://github.com/MatteoGys/Ling3/releases"><img alt="macOS Intel" src="https://img.shields.io/badge/macOS-Intel%20(x86__64)-555555"></a>
    <a href="https://github.com/MatteoGys/Ling3/blob/LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green"></a>
  </p>
</div>

---

## A propos

Ling3 est un outil d'analyse de fichiers TextGrid (Praat) pense pour le travail linguistique. L'application permet la detection de chevauchements, la preparation et la conversion de fichiers, le comptage de tokens, la generation d'un concordancier sur mesure ainsi que l'etude du contexte statistique d'un pivot.

---

## Fonctionnalites des modules

### 1. Parametres
Avant d'utiliser les outils d'analyse, l'onglet Parametres permet de configurer :
- **Fichiers TextGrid** : Le ou les fichiers a traiter (.TextGrid ou .txt), requis pour tous les onglets d'analyse.
- **Fichier Excel** : Le classeur (.xlsx) destinÃ© a recevoir les tableaux exportes (Tokenisation, Concordancier, Cooccurrences).
- **Separateur** : Caractere utilise pour distinguer plusieurs pivots saisis dans un meme champ (par defaut, une barre oblique inversee `\`).

### 2. Chevauchements
Verifie qu'aucun intervalle annote ne chevauche un intervalle d'une autre tier au sein d'un meme fichier TextGrid. En cas de conflit, l'application liste la tier source, le numero de l'intervalle, son timecode de fin, son texte et les tiers en cause pour faciliter la localisation.

### 3. Modifications
Regroupe les operations de preparation des fichiers :
- **Conversion TRS vers TextGrid** : Transforme des fichiers Transcriber (.trs, format CFPP) en fichiers TextGrid enregistres sur le Bureau (base sur l'algorithme LingTools de l'Universite de l'Oregon).
- **Remplacement de texte et fusion des silences** : Permet de modifier une chaine de caracteres sur l'ensemble d'un fichier et de fusionner les intervalles de silence consecutifs.
- **Extraction par mots pivots** : Isole dans un nouveau fichier les intervalles contenant un pivot specifique, tout en fusionnant les intervalles adjacents en silence.

### 4. Tokenisation
Comptabilise chaque token (mot) present dans les fichiers TextGrid et restitue les occurrences sous forme de tableau interactif. Offre des fonctionnalites de recherche instantanee, de tri par colonne et d'export direct vers Excel.

### 5. Concordancier
Genere un concordancier sur mesure dans un fichier Excel en extrayant chaque occurrence d'un pivot entouree de son contexte gauche et droit. Il est possible de configurer l'ordre des colonnes, le nombre de mots de contexte, ainsi que d'ajouter des metadonnees avancees (timecodes, duree, position par rapport aux silences).

### 6. Cooccurrences
Etudie l'environnement immediat d'un pivot en denombrant les mots presents a une distance definie (frequentiels ou fenetres contextuelles precises). Le resultat presente le type de contexte associe et le nombre d'occurrences globales ou par tier, exportable sous format Excel.

### Licence
Ce projet est distribue sous les termes de la licence mentionnee dans le fichier LICENSE. Pour toute question ou signalement de bug, contactez : matteo.gyselinck@student.outlook.com

