# Complete the Core class and constant surface

- [x] Compare the shared symbol catalog with the pinned PHP baseline.
- [x] Register and test portable Core constants in native code and opaque eval.
- [x] Implement target, build, installation, and executable constants with truthful Elephc values.
- [x] Complete enum interfaces and their shared catalog contracts.
- [x] Implement missing throwable classes, including ErrorException's extended constructor.
- [x] Implement predefined attribute classes and their validation and runtime behavior.
- [x] Implement SensitiveParameterValue and sensitive backtrace redaction.
- [x] Implement Serializable dispatch in serialization and unserialization.
- [x] Implement WeakReference and WeakMap with non-owning object edges and GC integration.
- [x] Add examples, regenerate builtin documentation, and run focused parity and target checks.

## Baseline and completion criteria

The pinned `scripts/docs/php_baseline.json` is PHP 8.5.10. At branch base
`24857c7717`, Core functions cover 62/62, class-likes cover 21/40, and constants
cover 37/89. Existing post-baseline extensions retain their version metadata.
Catalog presence alone does not establish behavioral compatibility. Each class
needs native and eval behavior, PHP-compatible validation, ownership tests where
relevant, and support for all five supported compilation targets.

## Missing class-likes

`AllowDynamicProperties`, `Attribute`, `BackedEnum`, `ClosedGeneratorException`,
`CompileError`, `DelayedTargetValidation`, `Deprecated`, `ErrorException`,
`NoDiscard`, `Override`, `ParseError`, `RequestParseBodyException`,
`ReturnTypeWillChange`, `SensitiveParameter`, `SensitiveParameterValue`,
`Serializable`, `UnitEnum`, `WeakMap`, and `WeakReference`.

Enum interface relationships already exist in parts of Magician; audit the
native checker, instance checks, Reflection, and catalog joins before changing
them. Attribute names already influence some checker and eval behavior, but
their classes are not yet available for construction and Reflection.

## Missing constants

Portable groups: `TRUE`, `FALSE`, `NULL`, `PHP_FLOAT_DIG`, the fourteen
`PHP_OUTPUT_HANDLER_*` flags, the eight `UPLOAD_ERR_*` codes, and the four
Zend debug/thread flags (`PHP_DEBUG`, `PHP_ZTS`, `ZEND_DEBUG_BUILD`,
`ZEND_THREAD_SAFE`). Boolean/null lookup must preserve PHP's special
case-insensitive names without relaxing ordinary constant case sensitivity.

Environment-dependent group: `DEFAULT_INCLUDE_PATH`, `PEAR_EXTENSION_DIR`,
`PEAR_INSTALL_DIR`, `PHP_BINARY`, `PHP_BINDIR`, `PHP_BUILD_DATE`,
`PHP_BUILD_PROVIDER`, `PHP_CLI_PROCESS_TITLE`, `PHP_CONFIG_FILE_PATH`,
`PHP_CONFIG_FILE_SCAN_DIR`, `PHP_DATADIR`, `PHP_EXTENSION_DIR`,
`PHP_FD_SETSIZE`, `PHP_LIBDIR`, `PHP_LOCALSTATEDIR`, `PHP_MANDIR`,
`PHP_MAXPATHLEN`, `PHP_PREFIX`, `PHP_SBINDIR`, `PHP_SHLIB_SUFFIX`,
`PHP_SYSCONFDIR`, and `ZEND_VM_KIND`. Values must describe Elephc rather than
copy paths or build metadata from the Docker image used for the PHP baseline.
`PHP_BINARY` must be resolved in the running program, never from the compiler
process. Document concepts that do not apply to a compiler without Zend.

## Verification

Use focused codegen and error tests, shared catalog/checker parity across all
targets, Magician unit tests, and selected PHP-oracle comparisons. Regenerate
the symbol registries and compatibility documentation through
`update-builtin-docs`; do not count placeholders as completed support.


## Completed verification

- Shared baseline contract gate: Core functions 62/62, class-likes 40/40, constants 89/89.
- Focused Core tests: 23 class tests, 5 constant tests, and 6 diagnostic tests.
- Native and opaque eval cover weak object and closure keys, identity, cloning, live iteration,
  sensitive traces, deferred attributes, executable-path aliases, and serialization hooks.
- Heap-debug regressions cover cycles, key resurrection, throwing value destructors, and
  callable and receiver owners extracted from locals widened by a future eval barrier.
- Existing callable ownership, ArrayAccess dispatch, and GC exception recovery regressions pass.
- Core declaration checking and capture ownership checks cover all five supported targets.
- Runtime assembly and representative programs assemble for every supported target. Focused
  Linux ARM64 programs also execute successfully with clean heap reports. Apple targets were
  verified by compilation and assembly, without native execution on this Linux host.
- The example matches PHP output and leaves a clean heap. Generated docs, site compatibility,
  builtin/EIR boundary, Rustdoc, assembly comments, and whitespace checks pass.

Existing eval restrictions on the standard stream constants and Reflection metadata retention
remain documented separately. The general property replacement ordering issue found during
WeakMap work is tracked as Code Journal task `21f22b33`.
