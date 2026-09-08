"""AstroBWTv3 mining difficulty target calculations."""

def compact_to_target(bits: int) -> int:
    """Convert compact target bits to a 256-bit integer target."""
    exponent = bits >> 24
    mantissa = bits & 0x007FFFFF
    if bits & 0x00800000:
        mantissa = -mantissa
    if exponent <= 3:
        return mantissa >> (8 * (3 - exponent))
    return mantissa << (8 * (exponent - 3))


def target_to_difficulty(target: int) -> float:
    """Calculate mining difficulty relative to Difficulty 1 target."""
    if target == 0:
        return 0.0
    # Difficulty 1 target: 0xffff * 2^208
    max_target = 0xFFFF * (2 ** 208)
    return max_target / float(target)
