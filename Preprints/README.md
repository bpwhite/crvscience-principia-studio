# Preprints for Principia Studio (`crvscience-principia-studio`)

**Author:** Bryan P. White (ORCID: [0000-0003-4191-3511](https://orcid.org/0000-0003-4191-3511))  
**Hub:** https://crvscience.com/apps

This directory contains the self-contained LaTeX source (`.tex`) and single-column PDF (`.pdf`) editions for both:

1. **Companion Application Preprint (`APP-PREPRINT-25`)**
   - **Title:** Principia Studio: Multithreaded Safe-Rust Computational Mathematics Explorer Across Six Domains and Nineteen Algorithmic Modules
   - **Files:** `principia-studio-computational-math.tex`, `principia-studio-computational-math.pdf`
   - **Canonical Zenodo DOI:** https://doi.org/10.5281/zenodo.24460709
   - **Figshare Mirror:** https://figshare.com/articles/preprint/Principia_Studio_Multithreaded_Safe-Rust_Computational_Mathematics_Explorer_Across_Six_Domains_and_Nineteen_Algorithmic_Modules/35091121
   - **Web Preprint:** https://crvscience.com/apps/preprints/principia-studio-computational-math

2. **CRV Science Public Apps Network Overview (`APP-PREPRINT-01` v1.4.0)**
   - **Title:** The CRV Science App Network: Twenty-Four Open-Source Rust Programs for Science, Data, and Simulation
   - **Files:** `crv-science-public-apps-network.tex`, `crv-science-public-apps-network.pdf`
   - **Canonical Zenodo DOI:** https://doi.org/10.5281/zenodo.24222398
   - **Figshare Mirror:** https://figshare.com/articles/preprint/The_CRV_Science_App_Network_Seventeen_Open-Source_Rust_Programs_for_Science_Data_and_Simulation/35013736
   - **Web Preprint:** https://crvscience.com/apps/preprints/crv-science-public-apps-network

## Compiling Standalone LaTeX

Both `.tex` files inline their bibliography so each compiles standalone with standard `pdflatex`:

```bash
pdflatex principia-studio-computational-math.tex
pdflatex crv-science-public-apps-network.tex
```
