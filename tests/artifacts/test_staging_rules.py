"""Test file for staging rules."""


def clean_function() -> str:
    """Return a clean string."""
    x: int = "not an int"  # type error - keeping same
    unused_var = 123  # unused variable - keeping same
    return "clean"
