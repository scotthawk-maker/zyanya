"""Cryptographic utilities and unit conversions for Zyanya."""

SOMPI_PER_ZYAN = 100_000_000  # 1 ZYAN = 10^8 sompi


def sompi_to_zyan(sompi: int) -> float:
    """Convert sompi (atomic unit) to ZYAN."""
    return sompi / SOMPI_PER_ZYAN


def zyan_to_sompi(zyan: float) -> int:
    """Convert ZYAN to sompi (atomic unit)."""
    return int(round(zyan * SOMPI_PER_ZYAN))


def validate_address(address: str, network: str = "mainnet") -> bool:
    """Validate a Zyanya Bech32 address prefix and format."""
    if not address or ":" not in address:
        return False

    prefix, payload = address.split(":", 1)
    expected_prefix = "zyanya" if network == "mainnet" else "zyanyatest"

    if prefix != expected_prefix:
        return False

    if len(payload) < 60:
        return False

    valid_charset = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    return all(c in valid_charset for c in payload.lower())
