from typing import Callable, overload


current_var = [0]
equations = []
plots = []


def next_var():
    var = current_var[0]
    current_var[0] += 1
    return var


def i(x: "Value|int") -> "Value":
    return Value(None, x) if isinstance(x, int) else x


def q(n: int, d: int) -> "RationalValue":
    return RationalValue(n, d)


# Types of Value:
# - integer constant: var = None, initial = int value
# - unknown constant (constant value bound by equations): var = int, initial = None
# - variable: var = int, initial = integer constant or unknown constant
class Value:
    def __init__(self, var: int | None, initial: "Value|int|None" = None):
        if var is None and (initial is None or isinstance(initial, Value)):
            raise Exception("Value without a var must have an integer initial")
        self.var = var
        self.initial = initial

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

    def integer_valued_operation(
        self, eq: Callable[["Value", "Value"], str], v: Callable[[int], int]
    ) -> "Value":
        if self.var is None:
            return Value(None, v(self.initial_as_int()))
        if self.initial is None:
            initial = None
        else:
            initial = i(self.initial).integer_valued_operation(eq, v).maybe_int()
        result = Value(next_var(), initial=initial)
        equations.append(eq(self, result))
        return result

    def non_integer_valued_operation(
        self, eq: Callable[["Value", "Value"], str]
    ) -> "Value":
        if self.var is None or self.initial is None:
            initial = None
        else:
            initial = i(self.initial).non_integer_valued_operation(eq).maybe_int()
        result = Value(next_var(), initial=initial)
        equations.append(eq(self, result))
        return result

    def __neg__(self) -> "Value":
        return self.integer_valued_operation(lambda a, b: f"{a} + {b}", lambda a: -a)

    def __pow__(self, power: "Value") -> "Value":
        if isinstance(power, RationalValue):
            return (
                self.non_integer_valued_operation(
                    lambda a, b: (
                        f"{a}^{power.n} - {b}^{power.d}"
                        if a.var is not None
                        else f"{a.initial_as_int() ** power.n} - {b}^{power.d}"
                    )
                )
                if power.n > 0
                else self.non_integer_valued_operation(
                    lambda a, b: (
                        f"1 - {a}^{-power.n}*{b}^{power.d}"
                        if a.var is not None
                        else f"1 - {a.initial_as_int() ** (-power.n)}*{b}^{power.d}"
                    )
                )
            )
        if power.var is not None:
            raise Exception("Only constant integer powers are supported")
        return (
            self.integer_valued_operation(
                lambda a, b: f"{a}^{power} - {b}", lambda a: a ** power.initial_as_int()
            )
            if power.initial_as_int() > 0
            else self.non_integer_valued_operation(
                lambda a, b: (
                    f"1 - {a}^{-power}*{b}"
                    if a.var is not None
                    else f"1 - {a.initial_as_int() ** (-power.initial_as_int())}*{b}"
                )
            )
        )

    def abs(self) -> "Value":
        return self.integer_valued_operation(
            lambda a, b: f"{a}^2 - {b}^2", lambda a: abs(a)
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
    ) -> "Value":
        if self.var is None and other.var is None:
            return Value(None, v(self.initial_as_int(), other.initial_as_int()))
        if self.initial is None or other.initial is None:
            initial = None
        else:
            initial = (
                i(self.initial)
                .integer_valued_binary_operation(i(other.initial), eq, v)
                .maybe_int()
            )
        result = Value(next_var(), initial=initial)
        equations.append(eq(self, other, result))
        return result

    def non_integer_valued_binary_operation(
        self,
        other: "Value",
        eq: Callable[["Value", "Value", "Value"], str],  # op1, op2, result
    ) -> "Value":
        arg1 = self
        arg2 = other
        if arg1.initial is None and arg2.initial is not None and arg2.var is not None:
            # Unknown constant (arg1) is elevated to a full-fledged Value
            arg1 = Value(arg1.var, initial=self)
        if arg2.initial is None and arg1.initial is not None and arg1.var is not None:
            # Unknown constant (arg2) is elevated to a full-fledged Value
            arg2 = Value(arg2.var, initial=other)
        if arg1.initial is None or arg2.initial is None:
            initial = None
        elif arg1.var is None and arg2.var is None:
            initial = None
        else:
            initial = (
                i(arg1.initial)
                .non_integer_valued_binary_operation(i(arg2.initial), eq)
                .maybe_int()
            )
        result = Value(next_var(), initial=initial)
        equations.append(eq(arg1, arg2, result))
        return result

    def __add__(self, other: "Value") -> "Value":
        return self.integer_valued_binary_operation(
            other, lambda a, b, c: f"{a} + {b} - {c}", lambda a, b: a + b
        )

    def __sub__(self, other: "Value") -> "Value":
        return self.integer_valued_binary_operation(
            other, lambda a, b, c: f"{a} - {b} - {c}", lambda a, b: a - b
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
            other, lambda a, b, c: f"{a}*{b} - {c}", lambda a, b: a * b
        )

    def __truediv__(self, other: "Value") -> "Value":
        return self.non_integer_valued_binary_operation(
            other, lambda a, b, c: f"{a} - {b}*{c}"
        )


class RationalValue(Value):
    def __init__(self, n: int, d: int):
        v = next_var()
        super().__init__(v, None)
        self.n = n
        self.d = d
        equations.append(f"{d}*{self} - {n}")


def sqrt(value: Value) -> Value:
    return value.non_integer_valued_operation(lambda a, b: f"{a} - {b}^2")


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
        super().__init__(Value(next_var(), initial=x), Value(next_var(), initial=y))


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
        super().__init__(Value(next_var(), initial=x), Value(next_var(), initial=y))


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


def plot(name: str, point: FreePoint) -> None:
    plots.append(f"{name} {point.x} {point.y}")
