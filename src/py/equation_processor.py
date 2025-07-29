import math
from typing import Callable, cast, overload


current_var = [0]
equations = []
plots = []
compute_float_initial = [False]


def next_var():
    var = current_var[0]
    current_var[0] += 1
    return var


def i(x: "Value|int") -> "Value":
    return (
        Value(None, initial=x, float_initial=maybe_float_initial(lambda: float(x)))
        if isinstance(x, int)
        else x
    )


def new_var(x: int) -> "Value":
    return Value(
        next_var(), initial=x, float_initial=maybe_float_initial(lambda: float(x))
    )


def q(n: int, d: int) -> "RationalValue":
    return RationalValue(n, d)


def maybe_float_initial(f: Callable[[], float]) -> float | None:
    if compute_float_initial[0]:
        return f()
    return None


# Types of Value:
# - integer constant: var = None, initial = int value
# - unknown constant (constant value bound by equations): var = int, initial = None
# - variable: var = int, initial = integer constant or unknown constant
class Value:
    def __init__(
        self,
        var: int | None,
        initial: "Value|int|None" = None,
        float_initial: float | None = None,
    ):
        if var is None and (initial is None or isinstance(initial, Value)):
            raise Exception("Value without a var must have an integer initial")
        self.var = var
        self.initial = initial
        self.float_initial = float_initial

    def __str__(self):
        if self.var is None:
            return str(self.initial)
        base = self.var % 26
        suffix = self.var // 26
        if suffix == 0:
            return chr(ord("a") + base)
        else:
            return f"{chr(ord('a') + base)}{suffix}"

    def __repr__(self):
        return f"Value(var={self.var}, initial={repr(self.initial)})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Value):
            return False
        return self.var == other.var and self.initial == other.initial

    def maybe_int(self) -> "Value|int":
        if self.var is None:
            if self.initial is None:
                raise Exception("Value without an initial is not an integer")
            return self.initial
        return self

    def initial_as_int(self) -> int:
        if not isinstance(self.initial, int):
            raise Exception(f"Integer initial value not found in {self}")
        return self.initial

    def float_initial_as_float(self) -> float:
        if self.float_initial is None:
            raise Exception(f"Float initial value not found in {self}")
        return self.float_initial

    def integer_valued_operation(
        self,
        eq: Callable[["Value", "Value"], str],
        v: Callable[[int], int],
        vf: Callable[[float], float],
    ) -> "Value":
        float_initial = maybe_float_initial(lambda: vf(self.float_initial_as_float()))
        if self.var is None:
            int_value = v(self.initial_as_int())
            return Value(None, initial=int_value, float_initial=float_initial)
        if self.initial is None:
            initial = None
        else:
            initial = i(self.initial).integer_valued_operation(eq, v, vf).maybe_int()
        result = Value(next_var(), initial=initial, float_initial=float_initial)
        equations.append(eq(self, result))
        return result

    def non_integer_valued_operation(
        self, eq: Callable[["Value", "Value"], str], vf: Callable[[float], float]
    ) -> "Value":
        float_initial = maybe_float_initial(lambda: vf(self.float_initial_as_float()))
        if self.var is None or self.initial is None:
            initial = None
        else:
            initial = i(self.initial).non_integer_valued_operation(eq, vf).maybe_int()
        result = Value(next_var(), initial=initial, float_initial=float_initial)
        equations.append(eq(self, result))
        return result

    def __neg__(self) -> "Value":
        return self.integer_valued_operation(
            lambda a, b: f"{a} + {b}", lambda a: -a, lambda a: -a
        )

    def __pow__(self, power: "Value") -> "Value":
        vf = lambda a: a ** power.float_initial_as_float()
        if isinstance(power, RationalValue):
            return (
                self.non_integer_valued_operation(
                    lambda a, b: (
                        f"{a}^{power.n} - {b}^{power.d}"
                        if a.var is not None
                        else f"{a.initial_as_int() ** power.n} - {b}^{power.d}"
                    ),
                    vf,
                )
                if power.n > 0
                else self.non_integer_valued_operation(
                    lambda a, b: (
                        f"1 - {a}^{-power.n}*{b}^{power.d}"
                        if a.var is not None
                        else f"1 - {a.initial_as_int() ** (-power.n)}*{b}^{power.d}"
                    ),
                    vf,
                )
            )
        if power.var is not None:
            raise Exception("Only constant integer powers are supported")
        return (
            self.integer_valued_operation(
                lambda a, b: f"{a}^{power} - {b}",
                lambda a: a ** power.initial_as_int(),
                vf,
            )
            if power.initial_as_int() > 0
            else self.non_integer_valued_operation(
                lambda a, b: (
                    f"1 - {a}^{-power}*{b}"
                    if a.var is not None
                    else f"1 - {a.initial_as_int() ** (-power.initial_as_int())}*{b}"
                ),
                vf,
            )
        )

    def abs(self) -> "Value":
        return self.integer_valued_operation(
            lambda a, b: f"{a}^2 - {b}^2", lambda a: abs(a), lambda a: abs(a)
        )

    # Valid combinations:
    # - constant + constant -> constant
    # - constant + unknown -> unknown (with equation)
    # - unknown + unknown -> unknown (with equation)
    # - constant + variable -> variable (with equation)
    # - variable + variable -> variable (with equation)
    def integer_valued_binary_operation(
        self,
        other: "Value",
        eq: Callable[["Value", "Value", "Value"], str],  # op1, op2, result
        v: Callable[[int, int], int],
        vf: Callable[[float, float], float],
    ) -> "Value":
        float_initial = maybe_float_initial(
            lambda: vf(self.float_initial_as_float(), other.float_initial_as_float())
        )
        if self.var is None and other.var is None:
            return Value(
                None,
                initial=v(self.initial_as_int(), other.initial_as_int()),
                float_initial=float_initial,
            )
        arg1 = self
        arg2 = other
        if arg1.initial is None and arg2.initial is not None and arg2.var is not None:
            # Unknown constant (arg1) is elevated to a full-fledged Value
            arg1 = Value(arg1.var, initial=arg1, float_initial=arg1.float_initial)
        if arg2.initial is None and arg1.initial is not None and arg1.var is not None:
            # Unknown constant (arg2) is elevated to a full-fledged Value
            arg2 = Value(arg2.var, initial=arg2, float_initial=arg2.float_initial)
        if arg1.initial is None or arg2.initial is None:
            initial = None
        else:
            initial = (
                i(arg1.initial)
                .integer_valued_binary_operation(i(arg2.initial), eq, v, vf)
                .maybe_int()
            )
        result = Value(next_var(), initial=initial, float_initial=float_initial)
        equations.append(eq(self, other, result))
        return result

    def non_integer_valued_binary_operation(
        self,
        other: "Value",
        eq: Callable[["Value", "Value", "Value"], str],  # op1, op2, result
        vf: Callable[[float, float], float],
    ) -> "Value":
        float_initial = maybe_float_initial(
            lambda: vf(self.float_initial_as_float(), other.float_initial_as_float())
        )
        arg1 = self
        arg2 = other
        if arg1.initial is None and arg2.initial is not None and arg2.var is not None:
            # Unknown constant (arg1) is elevated to a full-fledged Value
            arg1 = Value(arg1.var, initial=arg1, float_initial=arg1.float_initial)
        if arg2.initial is None and arg1.initial is not None and arg1.var is not None:
            # Unknown constant (arg2) is elevated to a full-fledged Value
            arg2 = Value(arg2.var, initial=arg2, float_initial=arg2.float_initial)
        if arg1.initial is None or arg2.initial is None:
            initial = None
        elif arg1.var is None and arg2.var is None:
            initial = None
        else:
            initial = (
                i(arg1.initial)
                .non_integer_valued_binary_operation(i(arg2.initial), eq, vf)
                .maybe_int()
            )
        result = Value(next_var(), initial=initial, float_initial=float_initial)
        equations.append(eq(arg1, arg2, result))
        return result

    def __add__(self, other: "Value") -> "Value":
        return self.integer_valued_binary_operation(
            other,
            lambda a, b, c: f"{a} + {b} - {c}",
            lambda a, b: a + b,
            lambda a, b: a + b,
        )

    def __sub__(self, other: "Value") -> "Value":
        return self.integer_valued_binary_operation(
            other,
            lambda a, b, c: f"{a} - {b} - {c}",
            lambda a, b: a - b,
            lambda a, b: a - b,
        )

    @overload
    def __mul__(self, other: "Vector") -> "Vector":
        pass

    @overload
    def __mul__(self, other: "Value") -> "Value":
        pass

    def __mul__(self, other: "Value|Vector") -> "Value|Vector":
        if isinstance(other, Vector):
            return other * self
        return self.integer_valued_binary_operation(
            other,
            lambda a, b, c: f"{a}*{b} - {c}",
            lambda a, b: a * b,
            lambda a, b: a * b,
        )

    def __truediv__(self, other: "Value") -> "Value":
        return self.non_integer_valued_binary_operation(
            other,
            lambda a, b, c: f"{a} - {b}*{c}",
            lambda a, b: a / b,
        )


class RationalValue(Value):
    def __init__(self, n: int, d: int):
        v = next_var()
        super().__init__(v, initial=None, float_initial=n / d)
        self.n = n
        self.d = d
        equations.append(f"{d}*{self} - {n}")


def sqrt(value: Value) -> Value:
    return value.non_integer_valued_operation(
        lambda a, b: f"{a} - {b}^2", lambda a: math.sqrt(a)
    )


class Point:
    def __init__(self, x: Value, y: Value):
        self.x = x
        self.y = y

    def __repr__(self):
        return f"Point(x={repr(self.x)}, y={repr(self.y)})"

    def __sub__(self, other: "Point") -> "Vector":
        return Vector(self.x - other.x, self.y - other.y)

    def __add__(self, other: "Vector") -> "Point":
        return Point(self.x + other.x, self.y + other.y)


class FixedPoint(Point):
    def __init__(self, x: int, y: int):
        super().__init__(i(x), i(y))


class FreePoint(Point):
    def __init__(self, x: int, y: int):
        super().__init__(new_var(x), new_var(y))


class Midpoint(Point):
    def __init__(self, point1: Point, point2: Point):
        super().__init__((point1.x + point2.x) / i(2), (point1.y + point2.y) / i(2))


class IntersectionPoint(Point):
    def __init__(self, line1: "Line", line2: "Line"):
        # Let line1 := (x - a) * n = 0, line2 := (x - b) * m = 0
        # Then with x = a + n' t, (a - b) * m + t (n' * m) = 0 => t = (b - a) * m / (n' * m)
        # intersection = a + n' * (b - a) * m / (n' * m)
        n_prime = line1.n.rotated90()
        factor = ((line2.o - line1.o) * line2.n) / (n_prime * line2.n)
        super().__init__(line1.o.x + n_prime.x * factor, line1.o.y + n_prime.y * factor)


class Projection(Point):
    def __init__(self, point: Point, line: "Line"):
        # proj = a - n ((a - p) * n) / (n * n) (p = point, (a, n) = line)
        factor = (point - line.o) * line.n / line.n.length_sqr()
        projection_vector = Vector(point.x, point.y) - line.n * factor
        super().__init__(projection_vector.x, projection_vector.y)


class Reflection(Point):
    def __init__(self, point: Point, line: "Line"):
        # reflection = a - 2 n ((a - p) * n) / (n * n) (p = point, (a, n) = line)
        factor = i(2) * ((point - line.o) * line.n) / line.n.length_sqr()
        reflection_vector = Vector(point.x, point.y) - line.n * factor
        super().__init__(reflection_vector.x, reflection_vector.y)


class ScaledVectorPoint(Point):
    def __init__(self, k: Value, point1: Point, point2: Point):
        # svp = point1 + k * (point2 - point1)
        super().__init__(
            point1.x + k * (point2.x - point1.x), point1.y + k * (point2.y - point1.y)
        )


class Vector:
    def __init__(self, x: Value, y: Value):
        self.x = x
        self.y = y

    def __repr__(self):
        return f"Vector(x={repr(self.x)}, y={repr(self.y)})"

    def rotated90(self) -> "Vector":
        return Vector(self.y, -self.x)

    def length_sqr(self) -> Value:
        return self * self

    def length(self) -> Value:
        return sqrt(self.length_sqr())

    def __add__(self, other: "Vector") -> "Vector":
        return Vector(self.x + other.x, self.y + other.y)

    def __sub__(self, other: "Vector") -> "Vector":
        return Vector(self.x - other.x, self.y - other.y)

    @overload
    def __mul__(self, other: "Vector") -> "Value":
        pass

    @overload
    def __mul__(self, other: "Value") -> "Vector":
        pass

    def __mul__(self, other: "Vector|Value") -> "Vector|Value":
        if isinstance(other, Value):
            return Vector(self.x * other, self.y * other)
        return self.x * other.x + self.y * other.y

    def __truediv__(self, other: Value) -> "Vector":
        return self * (i(1) / other)


class FixedVector(Vector):
    def __init__(self, x: int, y: int):
        super().__init__(i(x), i(y))


class FreeVector(Vector):
    def __init__(self, x: int, y: int):
        super().__init__(new_var(x), new_var(y))


class Line:
    def __init__(self, o: Point, n: Vector):
        self.o = o
        self.n = n

    def contains(self, p: Point):
        v = (p - self.o) * self.n
        equations.append(f"{v}")
        if v.initial is not None:
            equations.append(f"{v.initial}")

    def distance_to_point_sqr(self, p: Point) -> Value:
        return ((p - self.o) * self.n) ** i(2) / self.n.length_sqr()

    def distance_to_point(self, p: Point) -> Value:
        return (p - self.o) * self.n / self.n.length()


class LineAB(Line):
    def __init__(self, a: Point, b: Point):
        super().__init__(a, (b - a).rotated90())


class PpBisector(Line):
    def __init__(self, a: Point, b: Point):
        super().__init__(a + (b - a) / i(2), b - a)


class PpToLine(Line):
    def __init__(self, point: Point, line: Line):
        super().__init__(point, line.n.rotated90())


class PlToLine(Line):
    def __init__(self, point: Point, line: Line):
        super().__init__(point, line.n)


def d(a: Point | Line, b: Point | Line) -> Value:
    if isinstance(a, Line) and isinstance(b, Point):
        return a.distance_to_point(b)
    if isinstance(a, Point) and isinstance(b, Line):
        return b.distance_to_point(a)
    if isinstance(a, Point) and isinstance(b, Point):
        return sqrt(d_sqr(a, b))
    raise Exception("d() cannot be called with two lines")


def d_sqr(a: Point | Line, b: Point | Line) -> Value:
    if isinstance(a, Line) and isinstance(b, Point):
        return a.distance_to_point_sqr(b)
    if isinstance(a, Point) and isinstance(b, Line):
        return b.distance_to_point_sqr(a)
    if isinstance(a, Point) and isinstance(b, Point):
        return (a.x - b.x) ** i(2) + (a.y - b.y) ** i(2)
    raise Exception("d() cannot be called with two lines")


def cot(a: Vector, b: Vector) -> Value:
    return (a * b) / (a.x * b.y - a.y * b.x)


def sqrt(x: Value) -> Value:
    return x ** q(1, 2)


def is_constant(x: Value):
    if x.var is None:
        return
    if x.initial is None:
        raise Exception('is_constant() call for an "unknown value" is not allowed')
    equations.append(f"{x} - {x.initial}")


def is_zero(x: Value):
    if isinstance(x.initial, int) and x.initial_as_int() != 0:
        raise Exception(f"is_zero() call for a non-zero constant {x} is not allowed")
    if x.var is None:
        return
    equations.append(f"{x}")
    if not isinstance(x.initial, int):
        equations.append(f"{x.initial}")


def is_zero_vector(v: Vector):
    is_zero(v.x)
    is_zero(v.y)


def plot(name: str, point: Point) -> None:
    plots.append(f"{name} {point.x} {point.y}")
