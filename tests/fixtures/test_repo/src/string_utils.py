"""String utility functions for testing semantic search across languages"""


def reverse_string(s: str) -> str:
    """Reverses a string

    Args:
        s: The string to reverse

    Returns:
        The reversed string
    """
    return s[::-1]


def count_words(text: str) -> int:
    """Counts the number of words in a text

    Args:
        text: The text to analyze

    Returns:
        Number of words
    """
    return len(text.split())


def is_palindrome(s: str) -> bool:
    """Checks if a string is a palindrome

    Args:
        s: The string to check

    Returns:
        True if palindrome, False otherwise
    """
    cleaned = ''.join(c.lower() for c in s if c.isalnum())
    return cleaned == cleaned[::-1]
