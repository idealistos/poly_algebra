# Variable elimination algorithm and discussion

## Summary

To eliminate variables from the system of algebraic equations to arrive at the single equation $F(x, y) = 0$ describing the curve, this project uses a non-standard technique of "naïve" elimination followed by factoring the resulting polynomial and detecting extraneous factors, which appeared due to multiplying with expressions that later turn out to be identically zero.

The motivation for this approach (rather than the standard one relying on the Gröbner basis) is because in practice it often has better performance. Also, the way the equations are produced from geometric relations leads to a high number of variables and sparse polynomials (meaning that a polynomial typically uses a small subset of variables), thus storing the polynomial as the list of monomials (typically used when computing the Gröbner basis) isn't well suited for this task.

## Details

The "naïve" elimination consists on focusing on one variable at a time. Given two polynomials containing this variable, one can always produce a polynomial that has this variable in a smaller degree, by multiplying the polynomials with appropriate factors and adding them together (to clarify: if the first polynomial has the variable $a$ in the degree $d_1$ and the second polynomial has this variable in the degree $0 < d_2 <= d_1$, the resulting polynomial will have the degree $d_3 < d_1$). This process leads to a polynomial that doesn't contain the variable.

However, even for small problems, this process leads to extraneous factors. To counter this, the resulting two-variable polynomial is factored, and we analyze which factors are indeed a part of the solution, and which factors are extraneous.

An idea how to achieve this is based on the fact that one can substitute $x = x(t)$ and $y = y(t)$ into the factor we examine $F_i(x, y) = 0$, which will then produce the polynomial $q(t)$. The roots of this polynomial lead to certain points $x$ and $y$ satisfying the equation $F_i(x, y) = 0$.

In many cases, the resulting equation $F(x, y) = 0$ was obtained by eliminating some other variable $z$ from two equations in x, y, and z, and the last step before $z$ was eliminated was the expression $G(x, y) z + H(x, y) = 0$. If this is the case, we can often construct $z(t)$ so that this relation holds for every root of $q(t)$, i.e., $G(x, y) z + H(x, y)$ modulo $q(t)$ is zero after subtituting $x = x(t)$ and $y = y(t)$. To ensure $G(x(t), y(t))$ is invertible (or zero), one switches the coefficients from $\mathbb{Z}$ to $\mathbb{F}_p$ with a large prime $p$. In the case both $G(x(t), y(t))$ and $H(x(t), y(t))$ are zero modulo $q(t)$, an arbitrary $z(t)$ satisfies the equation, thus it is chosen randomly.

In this implementation, $x(t)$ and $y(t)$ are chosen as linear polynomials with small random coefficients, checked to be linearly independent.

Proceeding in this manner, one gets polynomials in $t$ for each variable of the original system. Substituting them back to original equations, one gets that

- either all expressions are 0 modulo $q(t)$, in which case $F_i(x, y)$ is a true factor (unless there was a rare coincidence regarding how $x(t)$, $y(t)$, and $p$ were chosen)
- or some expression is not identical to 0, in which case $F_i(x, y)$ is definitely an extraneous factor

## Issues with this approach

There are cases when the last expression before $z$ was eliminated had $z$ in some degree above one. An example:

$$
\begin{array}{l}
u^2 = 25 \\

u^2 - 2 u z + z^2 = 25 \\

u^2 - 2 u v + v^2 - x^2 - 6x - y^2 = 9 \\

v^2 - x^2 - y^2 + 6x = 9
\end{array}
$$


The last step before all variables other than $x$ and $y$ are eliminated was

$$
\begin{array}{l}
16z^2 - z^2 x^2 - z^2 y^2 + 36x^2 = 0 \\

z^2 x^2 = 100x^2
\end{array}
$$

and it contains only $z^2$, but not just $z$. However, one needs $z$ for finding $v$ from the expression

$$
18z - z^2 v + 2zy^2 - 12zx + 12vx + 2zx^2 = 0.
$$

I haven't found a solution so far :)


