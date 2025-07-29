import sys
import os
import traceback

# Add parent directory to path so we can import equation_processor
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from equation_processor import (
    FixedPoint,
    FreePoint,
    LineAB,
    d_sqr,
    i,
    is_constant,
    d,
    Value,
    equations,
    current_var,
    next_var,
    sqrt,
    Vector,
)


def run_tests():
    failures = []

    def assert_equal(actual, expected, test_name):
        if actual != expected:
            failures.append(f"{test_name}: expected {expected}, got {actual}")

    def assert_true(condition, test_name):
        if not condition:
            failures.append(f"{test_name}: expected True, got False")

    def assert_equation_exists(equation, test_name):
        if equation not in equations:
            failures.append(
                f"{test_name}: equation '{equation}' not found in {equations}"
            )

    # Test variable naming
    try:
        assert_equal(str(Value(0)), "a", "Variable name for index 0")
        assert_equal(str(Value(25)), "z", "Variable name for index 25")
        assert_equal(str(Value(26)), "a1", "Variable name for index 26")
        assert_equal(str(Value(51)), "z1", "Variable name for index 51")
        assert_equal(str(Value(52)), "a2", "Variable name for index 52")
    except Exception as e:
        failures.append(
            f"Variable naming test failed: {str(e)}\n{traceback.format_exc()}"
        )

    # Test FixedPoint
    try:
        p = FixedPoint(1, 2)
        assert_equal(p.x, Value(None, initial=1), "FixedPoint.x")
        assert_equal(p.y, Value(None, initial=2), "FixedPoint.y")
    except Exception as e:
        failures.append(f"FixedPoint test failed: {str(e)}\n{traceback.format_exc()}")

    # Test FreePoint
    try:
        p = FreePoint(3, 4)
        assert_equal(p.x, Value(current_var[0] - 2, initial=3), "FreePoint.x")
        assert_equal(p.y, Value(current_var[0] - 1, initial=4), "FreePoint.y")
    except Exception as e:
        failures.append(f"FreePoint test failed: {str(e)}\n{traceback.format_exc()}")

    # Test arithmetic operations with Variable
    try:
        # Clear equations for testing
        equations.clear()

        # Test addition
        v1 = Value(next_var(), initial=5)
        v2 = Value(next_var(), initial=3)
        result = v1 + v2
        assert_equal(
            result, Value(current_var[0] - 1, initial=8), "Variable addition value"
        )
        assert_equation_exists(f"{v1} + {v2} - {result}", "Variable addition equation")

        # Test subtraction
        result = v1 - v2
        assert_equal(
            result, Value(current_var[0] - 1, initial=2), "Variable subtraction value"
        )
        assert_equation_exists(
            f"{v1} - {v2} - {result}", "Variable subtraction equation"
        )

        # Test multiplication
        result = v1 * v2
        assert_equal(
            result,
            Value(current_var[0] - 1, initial=15),
            "Variable multiplication value",
        )
        assert_equation_exists(
            f"{v1}*{v2} - {result}", "Variable multiplication equation"
        )

        # Test division
        result = v1 / v2
        assert_equal(
            result,
            Value(current_var[0] - 1, initial=Value(current_var[0] - 2)),
            "Variable division value",
        )
        assert_equation_exists(f"{v1} - {v2}*{result}", "Variable division equation")
        assert_equation_exists(f"5 - 3*{result.initial}", "Variable division equation")

        # Test operations with integers
        result = v1 + i(2)
        assert_equal(
            result, Value(current_var[0] - 1, initial=7), "Variable + int value"
        )
        assert_equation_exists(f"{v1} + 2 - {result}", "Variable + int equation")

        result = i(2) + v1
        assert_equal(
            result, Value(current_var[0] - 1, initial=7), "int + Variable value"
        )
        assert_equation_exists(f"2 + {v1} - {result}", "int + Variable equation")

        result = v1 / i(2)
        assert_equal(
            result,
            Value(current_var[0] - 1, initial=Value(current_var[0] - 2)),
            "Variable / int value",
        )
        assert_equation_exists(f"{v1} - 2*{result}", "Variable / int equation")

        result = i(2) / i(3)
        assert_equal(
            result,
            Value(current_var[0] - 1),
            "int / int value",
        )
        assert_equation_exists(f"2 - 3*{result}", "int / int equation")

        result = -v1
        assert_equal(
            result,
            Value(current_var[0] - 1, initial=-5),
            "Variable negation value",
        )
        assert_equation_exists(f"{v1} + {result}", "Variable negation equation")

        result = v1 ** i(2)
        assert_equal(
            result,
            Value(current_var[0] - 1, initial=25),
            "Variable power value",
        )
        assert_equation_exists(f"{v1}^{2} - {result}", "Variable power equation")

        result = sqrt(v1)
        assert_equal(
            result,
            Value(current_var[0] - 1, initial=Value(current_var[0] - 2)),
            "Variable sqrt value",
        )
        assert_equation_exists(f"{v1}^1 - {result}^2", "Variable sqrt equation")
        assert_equation_exists(f"5 - {result.initial}^2", "Variable sqrt equation")

    except Exception as e:
        failures.append(
            f"Arithmetic operations test failed: {str(e)}\n{traceback.format_exc()}"
        )

    # Test is_constant
    try:
        p1 = FreePoint(0, 0)
        p2 = FreePoint(1, 2)
        is_constant(d_sqr(p1, p2))
        v = current_var[0] - 1
        assert_equation_exists(f"{p1.x} - {p2.x} - {Value(v - 4)}", "is_constant test")
        assert_equation_exists(f"{Value(v - 4)}^2 - {Value(v - 3)}", "is_constant test")
        assert_equation_exists(f"{p1.y} - {p2.y} - {Value(v - 2)}", "is_constant test")
        assert_equation_exists(f"{Value(v - 2)}^2 - {Value(v - 1)}", "is_constant test")
        assert_equation_exists(
            f"{Value(v - 3)} + {Value(v - 1)} - {Value(v)}", "is_constant test"
        )  # i = v, h = v-1, g = v-2, f = v-3, e = v-4
        assert_equation_exists(f"{Value(v)} - 5", "is_constant test")
    except Exception as e:
        failures.append(f"is_constant test failed: {str(e)}\n{traceback.format_exc()}")

    # Test vector operations
    try:
        # Clear equations for testing
        equations.clear()

        # Test point difference (creates vector)
        p1 = FixedPoint(1, 2)
        p2 = FixedPoint(3, 4)
        v1 = p2 - p1  # Vector from p1 to p2
        assert_equal(v1.x, Value(None, 2), "Point difference x component")
        assert_equal(v1.y, Value(None, 2), "Point difference y component")

        # Test vector addition
        v2 = Vector(Value(None, 1), Value(None, 1))
        v3 = v1 + v2
        assert_equal(v3.x, Value(None, 3), "Vector addition x component")
        assert_equal(v3.y, Value(None, 3), "Vector addition y component")

        # Test vector subtraction
        v4 = v1 - v2
        assert_equal(v4.x, Value(None, 1), "Vector subtraction x component")
        assert_equal(v4.y, Value(None, 1), "Vector subtraction y component")

        # Test vector-scalar multiplication
        scalar = Value(None, 2)
        v5 = v1 * scalar
        assert_equal(v5.x, Value(None, 4), "Vector-scalar multiplication x component")
        assert_equal(v5.y, Value(None, 4), "Vector-scalar multiplication y component")

        # Test scalar-vector multiplication (reverse order)
        v6 = scalar * v1
        assert_equal(v6.x, Value(None, 4), "Scalar-vector multiplication x component")
        assert_equal(v6.y, Value(None, 4), "Scalar-vector multiplication y component")

        # Test vector-vector multiplication (dot product)
        v7 = Vector(Value(None, 1), Value(None, 0))
        v8 = Vector(Value(None, 0), Value(None, 1))
        dot_product = v7 * v8
        assert_equal(dot_product, Value(None, 0), "Vector dot product (perpendicular)")

        v9 = Vector(Value(None, 2), Value(None, 3))
        v10 = Vector(Value(None, 4), Value(None, 5))
        dot_product2 = v9 * v10
        assert_equal(dot_product2, Value(None, 23), "Vector dot product (2*4 + 3*5)")

        # Test vector operations with variables
        equations.clear()

        # Create vectors with variable components
        var_x1 = Value(next_var(), initial=1)
        var_y1 = Value(next_var(), initial=2)
        var_x2 = Value(next_var(), initial=3)
        var_y2 = Value(next_var(), initial=4)

        v_var1 = Vector(var_x1, var_y1)
        v_var2 = Vector(var_x2, var_y2)

        # Test vector addition with variables
        v_sum = v_var1 + v_var2
        assert_equal(
            v_sum.x, Value(current_var[0] - 2, initial=4), "Variable vector addition x"
        )
        assert_equal(
            v_sum.y, Value(current_var[0] - 1, initial=6), "Variable vector addition y"
        )
        assert_equation_exists(
            f"{var_x1} + {var_x2} - {v_sum.x}", "Variable vector addition equation x"
        )
        assert_equation_exists(
            f"{var_y1} + {var_y2} - {v_sum.y}", "Variable vector addition equation y"
        )

        # Test vector-scalar multiplication with variables
        var_scalar = Value(next_var(), initial=2)
        v_scaled = v_var1 * var_scalar
        assert_equal(
            v_scaled.x,
            Value(current_var[0] - 2, initial=2),
            "Variable vector-scalar multiplication x",
        )
        assert_equal(
            v_scaled.y,
            Value(current_var[0] - 1, initial=4),
            "Variable vector-scalar multiplication y",
        )
        assert_equation_exists(
            f"{var_x1}*{var_scalar} - {v_scaled.x}",
            "Variable vector-scalar multiplication equation x",
        )
        assert_equation_exists(
            f"{var_y1}*{var_scalar} - {v_scaled.y}",
            "Variable vector-scalar multiplication equation y",
        )

        # Test vector dot product with variables
        dot_var = v_var1 * v_var2
        assert_equal(
            dot_var,
            Value(current_var[0] - 1, initial=11),
            "Variable vector dot product",
        )
        assert_equation_exists(
            f"{var_x1}*{var_x2} - {Value(current_var[0] - 3)}",
            "Variable vector dot product equation",
        )
        assert_equation_exists(
            f"{var_y1}*{var_y2} - {Value(current_var[0] - 2)}",
            "Variable vector dot product equation",
        )
        assert_equation_exists(
            f"{Value(current_var[0] - 3)} + {Value(current_var[0] - 2)} - {dot_var}",
            "Variable vector dot product equation",
        )

        # Test point difference with variable points
        equations.clear()
        free_p1 = FreePoint(0, 0)
        free_p2 = FreePoint(1, 1)
        v_diff = free_p2 - free_p1
        assert_equal(
            v_diff.x, Value(current_var[0] - 2, initial=1), "FreePoint difference x"
        )
        assert_equal(
            v_diff.y, Value(current_var[0] - 1, initial=1), "FreePoint difference y"
        )
        assert_equation_exists(
            f"{free_p2.x} - {free_p1.x} - {v_diff.x}", "FreePoint difference equation x"
        )
        assert_equation_exists(
            f"{free_p2.y} - {free_p1.y} - {v_diff.y}", "FreePoint difference equation y"
        )

        # Test vector rotation (90 degrees)
        v_rot = Vector(i(1), i(0))
        v_rotated = v_rot.rotated90()
        assert_equal(v_rotated.x, Value(None, 0), "Vector rotation x component")
        assert_equal(v_rotated.y, Value(None, -1), "Vector rotation y component")

        # Test vector length
        equations.clear()
        v_length = Vector(i(3), i(4))
        length = v_length.length()
        assert_equal(
            length,
            Value(current_var[0] - 1),
            "Vector length",
        )
        assert_equation_exists(
            f"25 - {length}^2",
            "Vector length equation",
        )

    except Exception as e:
        failures.append(
            f"Vector operations test failed: {str(e)}\n{traceback.format_exc()}"
        )

    return failures


scenarios = []


def scenario1():
    lineA = LineAB(FixedPoint(-1, 0), FixedPoint(1, 0))
    X = FreePoint(0, 2)
    is_constant(d(X, lineA))


scenarios.append(scenario1)

if __name__ == "__main__":
    failures = run_tests()

    for scenario in scenarios:
        try:
            equations.clear()
            current_var[0] = 0
            scenario()
        except Exception as e:
            failures.append(
                f"Scenario {scenario.__name__} failed: {str(e)}\n{traceback.format_exc()}"
            )
    if len(failures) == 0:
        print("All scenarios passed!")

    if failures:
        print("Test failures:")
        for failure in failures:
            print(f"  {failure}")
        sys.exit(1)
    else:
        print("All tests passed!")
