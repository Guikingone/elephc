"""Public API for Elephc's fail-closed php-src/WASM oracle foundation."""

from .aggregate import (
    AGGREGATE_SCHEMA,
    AggregateError,
    AggregateResult,
    aggregate_exact,
    load_capture_record,
)
from .capture import (
    CAPTURE_SCHEMA,
    MODULE_STATUS_FD_ENV,
    CaptureError,
    CaptureRecord,
    CaptureRequest,
    Normalization,
    RawBytes,
    capture_process,
)
from .comparator import (
    ComparisonError,
    ComparisonResult,
    Difference,
    compare_records,
)
from .contract import (
    EXECUTION_CELLS,
    CompilerArtifactProvenance,
    ContractError,
    OracleContract,
    PhpSrcPin,
    RunKey,
    RuntimeProvenance,
    SUPPORTED_PROFILES,
    SUPPORTED_RUNTIMES,
    sha256_bytes,
    sha256_file,
)

__all__ = [
    "AGGREGATE_SCHEMA",
    "CAPTURE_SCHEMA",
    "MODULE_STATUS_FD_ENV",
    "AggregateError",
    "AggregateResult",
    "CaptureError",
    "CaptureRecord",
    "CaptureRequest",
    "CompilerArtifactProvenance",
    "ComparisonError",
    "ComparisonResult",
    "ContractError",
    "Difference",
    "EXECUTION_CELLS",
    "Normalization",
    "OracleContract",
    "PhpSrcPin",
    "RawBytes",
    "RunKey",
    "RuntimeProvenance",
    "SUPPORTED_PROFILES",
    "SUPPORTED_RUNTIMES",
    "aggregate_exact",
    "capture_process",
    "compare_records",
    "load_capture_record",
    "sha256_bytes",
    "sha256_file",
]
