# Forgum Credits & Academic Inspirations (`CREDITS.md`)

This document formally acknowledges the foundational published literature, mathematical algorithms, animation principles, and artistic works that inspired and informed the technical architecture of **Forgum**.

---

## 1. Classical 2D Animation & Quadruped Gait Kinematics

### A. The Animator's Survival Kit (Richard Williams)
- **Author**: Richard Williams (Director of Animation for *Who Framed Roger Rabbit*, recipient of three Academy Awards)
- **Publication**: *The Animator's Survival Kit: A Manual of Methods, Principles and Formulas for Classical, Computer, Games, Stop Motion and Internet Animators* (Faber & Faber, London / New York, 2001; Expanded Edition, 2009).
- **Inspiration in Forgum**:
  - **8-Phase Locomotion Cycle**: Formulation of the contact phase (heel/paw strike), down/recoil position (maximum knee flexion & center-of-mass dip), passing position (swing leg clears ground, pelvic lift), and high-point / push-off position (maximum extension).
  - **Quadruped Stride Decoupling**: Separation of foreleg and hindleg gait phases by $90^\circ$ ($\Delta \phi = 0.25$ stride lag), accurately capturing mammalian lateral sequence walking.
  - **Articulated Leg Glyphs**: Structural transformation across knee, hock, ankle, and paw contact zones (`|`, `/`, `\`, `_`, `) `, `( `, `J`, `L`, `d`, `b`).
  - **Stride-Velocity Coupling**: Exact synchronization of linear ground displacement $\Delta x = v_x \cdot \Delta t$ with foot contact duration, eliminating the "treadmill slipping" / "cheap character swap" artifact.

### B. Disney's 12 Basic Principles of Animation (Frank Thomas & Ollie Johnston)
- **Authors**: Frank Thomas and Ollie Johnston (Disney Legend animators, "Nine Old Men")
- **Publication**: *Disney Animation: The Illusion of Life* (Abbeville Press, New York, 1981).
- **Inspiration in Forgum**:
  - **Principle 1: Squash and Stretch**: Continuous volumetric conservation ($H \cdot W \approx \text{const}$) where animal flanks expand horizontally during downward recoil and compress vertically during upward push-off.
  - **Principle 3: Staging & Bounded Height**: Viewport-adaptive mascot framing ensuring characters fit harmoniously within limited terminal heights without visual clipping.
  - **Principle 5: Follow-Through and Overlapping Action**: Caudal zone (tail sway) and appendages lagging behind thoracic movement with a phase delay ($\Delta \phi \approx \frac{\pi}{3}$) governed by pendulum wave equations.
  - **Principle 8: Secondary Action**: Cranial nodding and chewing/breathing oscillations that enrich the primary walking gait without obscuring it.

---

## 2. Procedural Botany, Plant Architecture & Nature Mathematics

### A. The Algorithmic Beauty of Plants (Przemysław Prusinkiewicz & Aristid Lindenmayer)
- **Authors**: Przemysław Prusinkiewicz and Aristid Lindenmayer
- **Publication**: *The Algorithmic Beauty of Plants* (The Virtual Laboratory, Springer-Verlag, New York, 1990).
- **Inspiration in Forgum**:
  - **Allometric Trunk Tapering**: Exponential and power-law trunk thickness equations $W(y) = W_{\text{base}} \cdot (1 - \alpha \frac{y}{H})^\beta$ modeling structural cambial growth from basal flairs to upper crown canopy junctions.
  - **22-Species Botanical Taxonomy**: Procedural structural envelopes modeling 22 distinct botanical archetypes across gymnosperms (pines, firs, cypresses), angiosperms (oaks, maples, birches, willows, acacias, baobabs), monocots (palms, bamboos), pteridophytes (ferns), and succulent xerophytes (cacti).
  - **Ecological Biome Association**: Mathematical probability distributions matching vegetative morphology to climatic biomes (e.g., cold boreal conifers, tropical mangroves/palms, desert succulents).

### B. Phyllotaxis Spiral Distribution (Helmut Vogel)
- **Author**: Helmut Vogel
- **Publication**: *A better way to construct the sunflower head* (Mathematical Biosciences, Vol. 44, Issues 3–4, pp. 179–189, 1979).
- **Inspiration in Forgum**:
  - **Golden Ratio Phyllotaxis**: Application of the golden angle $\alpha = 360^\circ \times (1 - \Phi^{-1}) \approx 137.507764^\circ$ ($r_k = c\sqrt{k}, \theta_k = k \cdot \alpha$) for distributing dynamic fruits (berries, apples, citrus) and floral star blossoms on tree crowns.
  - **Window-Width Non-Repetition**: Zero spatial clumping and mathematically guaranteed unique fruit/blossom patterns across any terminal window width.

### C. Low-Discrepancy Sequences (Hermann Weyl)
- **Author**: Hermann Weyl
- **Publication**: *Über die Gleichverteilung von Zahlen mod. Eins* (Mathematische Annalen, Vol. 77, pp. 313–352, 1916).
- **Inspiration in Forgum**:
  - **Weyl Low-Discrepancy Sequencing**: The sequence $x_n = (n \cdot \Phi^{-1} + \sigma) \pmod 1$ provides quasi-random, perfectly uniform flora species sequencing without artificial clustering or cyclic repetition across arbitrary terminal columns.

---

## 3. Atmospheric Perspective, Shading & Terrain Synthesis

### A. Treatise on Painting / Trattato della pittura (Leonardo da Vinci)
- **Author**: Leonardo da Vinci
- **Manuscripts**: Compiled c. 1490–1519; published posthumously by Raphaël Trichet du Fresne, Paris, 1651.
- **Inspiration in Forgum**:
  - **Atmospheric Perspective (*Prospettiva de' perdimenti*)**: Optical attenuation of contrast and progressive shifting toward cooler, desaturated tones (slate/cyan sfumato) as landscape features recede toward the horizon.
  - **Three-Tier Depth Strata**:
    1. *Far Horizon (Atmospheric Haze)*: Low-contrast, desaturated mist tones (`#37474f`, `#455a64`) and soft textures.
    2. *Mid-Ground Ridges (Strata & Shadows)*: Medium-contrast geological bedding and shadow crevasses (`#546e7a`, `#607d8b`).
    3. *Foreground Slopes (Direct Illumination)*: High-contrast directional illumination, crisp snowcaps, and deep rock facets (`#78909c`, `#ffffff`).

### B. Procedural Terrain & Multi-Harmonic Superposition (F. Kenton Musgrave & Ken Perlin)
- **Authors**: F. Kenton Musgrave, Ken Perlin, Stefan Gustavson
- **Publications**:
  - F. K. Musgrave, *Fractal Terrains* in *Texturing and Modeling: A Procedural Approach* (Morgan Kaufmann, 1994).
  - Ken Perlin, *An Image Synthesizer* (ACM SIGGRAPH Computer Graphics, Vol. 19, No. 3, pp. 287–296, 1985).
- **Inspiration in Forgum**:
  - **Harmonic Sinusoidal Superposition**: The continuous mountain elevation function $H(x) = H_{\text{base}} \cdot \left[1 + \sum_{k=1}^M A_k |\sin(2\pi k x / \lambda + \phi_k)|^\gamma\right]$ synthesizing realistic alpine peaks, volcanic calderas, and eroded valleys with zero runtime heap allocation.

---

## 5. Color Vision Deficiency (CVD) Simulation, Daltonization & Accessible Design

### A. Dichromatic Simulation in LMS Cone Fundamentals (Hans Brettel, Françoise Viénot & John D. Mollon)
- **Authors**: Hans Brettel, Françoise Viénot, and John D. Mollon
- **Publication**: *Computerized simulation of color appearance for dichromats* (Journal of the Optical Society of America A, Vol. 14, Issue 10, pp. 2647–2655, 1997).
- **Inspiration in Forgum**:
  - **LMS Cone Transformation**: Transformation of non-linear sRGB values via gamma expansion into linear tristimulus coordinates, projecting onto protanopic, deuteranopic, and tritanopic copunctal planes in LMS space.
  - **Dichromatic Inspection Modes**: High-fidelity LMS simulation filters (`protanopia_sim`, `deuteranopia_sim`, `tritanopia_sim`) enabling designers to inspect terminal scenes exactly as perceived by dichromats.

### B. Physiologically-Based Daltonization & Spectral Redistribution (Gustavo M. Machado et al.)
- **Authors**: Gustavo M. Machado, Manuel M. Oliveira, and Leandro A. F. Fernandes
- **Publication**: *A Physiologically-based Model for Simulation of Color Vision Deficiency* (IEEE Transactions on Visualization and Computer Graphics, Vol. 15, No. 6, pp. 1291–1298, 2009).
- **Inspiration in Forgum**:
  - **Daltonization Error Re-projection**: Calculation of lost chromatic energy $\mathbf{\Delta} = \mathbf{C}_{\text{linear}} - \mathbf{C}_{\text{sim}}$ and dynamic compensatory shifting into preserved receptor channels (e.g., shifting lost red/green contrast into high-contrast blue/amber pathways).
  - **Active Accessibility Modes**: Real-time Daltonization filters (`protanopia`, `deuteranopia`, `tritanopia`) ensuring all mascot coats, botanical foliage, fruits, and road textures maintain distinct visual contrast.

### C. Color Universal Design Principles (Masataka Okabe & Kei Ito)
- **Authors**: Masataka Okabe and Kei Ito
- **Publication**: *Color Universal Design (CUD) - How to make figures and presentations that are friendly to Colorblind People* (J*Fly Data Depository, University of Tokyo, 2002; revised 2008).
- **Inspiration in Forgum**:
  - **Calibrated Soothing Nature Palettes**: Utilization of high-discriminability hues, soft sage greens, warm honey citruses, dusty roseberries, and elderberry plums calibrated for universal legibility without visual fatigue.
  - **Dual Coding**: Combining color cues with unique morphological ASCII characters and distinct topological shapes so no visual feature relies exclusively on color discernment.

---

## 6. Terminal ASCII Art Heritage & Community

- **Tony Monroe**: Creator of the original Unix `cowsay` utility (1999), establishing the ubiquitous `$the_cow` heredoc standard, speech bubbles, and placeholder conventions.
- **Classic ASCII Artists**:
  - **Joan G. Stark (`jgs`)**: Pioneer of miniature animal and floral ASCII silhouettes.
  - **Hayley Jane Wakenshaw (`hjw`)**: Renowned contributor of expressive felines and wildlife ASCII compositions.
  - **Shanaka Dias (`snd`)**: Renowned master of compact animal and big cat character art.
  - **Felix Lee**: Renowned contributor of early detailed animal silhouettes.
  - The international text-mode art archive community ([asciiart.eu](https://www.asciiart.eu/)).

---

*Forgum honors these mathematical foundations, animation pioneers, and artistic traditions, bringing static terminal text to life with computational rigor.*

