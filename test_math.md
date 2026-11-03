# Cedilla Math Rendering Test

## 1. Greek Letters and Variables (Display Math)
$$\alpha, \quad \beta, \quad \delta, \quad \gamma, \quad \epsilon, \quad \nu, \quad \theta$$

## 2. Subscripts, Text and Equations
$$a_1, \quad a_2, \quad v_{\text{initial}}$$

## 3. Inline Math
This is an inline equation: $E = mc^2$ and here is another variable $k$ within the paragraph.

## 4. LaTeX Delimiters \( ... \) and \[ ... \]
Inline LaTeX: \( \sum_{i=1}^n i = \frac{n(n+1)}{2} \)

Display LaTeX:
\[
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
\]

## 5. Direct Typst Syntax in Math Mode
$$typ: cases(1 "if" x > 0, 0 "if" x = 0, -1 "if" x < 0)$$

## 6. Variables without Backslashes & Delimiters with Spaces
$ mc^2 $

\(mc^2\)

(\gamma) -> \(\gamma\)

\[mc^2\]

\[\beta\]

$$mc^2$$

$\alpha$

$\sqrt{2}$

$$\sqrt{2}$$

$typ: mc^2$

$$typ: a/b$$

