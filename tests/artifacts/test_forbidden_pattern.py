"""Test file with forbidden pattern."""


def bad_function() -> None:
    x: int = "string"  # type: ignore
    print(x)
