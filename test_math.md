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

## 7. Typst Block Equations with Single Dollar ($ ... $)
$ A = pi r^2 $

$
sum_(i=1)^n i = (n(n+1)) / 2
$

## 8. Escaped Currency (\$): No Equation Trigger
El café vale \$5 y el sándwich vale \$10.
El producto tiene un descuento de \$2.50.

## 9. Equations Starting with Numbers ($5-x$)
Formula inline: $5-x$ y también $2x + 1 = 0$.

## 10. Absolute Values, Norms and Pipes (|)
Typst nativo: $abs(x) < 5$ y norma: $norm(v) = 1$.
Relación Typst: $x mid y$.
LaTeX absoluto: $\left| x \right| \le 1$.
LaTeX norma: $\left\| v \right\|$.
LaTeX barra relacional: $a \mid b$.
Matriz con barras (determinante): $mat(delim: "|", 1, 2; 3, 4)$


