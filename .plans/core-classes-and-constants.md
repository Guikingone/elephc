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
- Focused Core tests: 24 class tests, 16 review regressions, 5 constant tests, and 7 diagnostic tests.
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

## PR 1870 review corrections

- [x] Fix and verify every distinct failure in the initial CI matrix.
- [x] Preserve nested serialization reference state and validate legacy hook results.
- [x] Accept tentative Serializable method signatures without mandatory return hints.
- [x] Warn for discarded eval instance method results without reevaluating receivers.
- [x] Regenerate builtin docs and run focused regression and target checks.

The follow-up also protects the eval error-handler C boundary, consumes thrown callable
boxes, and tests callback exceptions with heap debugging. Local verification includes
all 95 serialization regressions, the example corpus, the initial CI failures, and
seven focused ARM64 executables. CI and Greptile outcomes for the published commit
are recorded on PR #1870 and in the shared Code Journal.

The follow-up ARM64 review found an ABI scratch-register clobber in nested hydration.
Load the sharing mode before taking the reference-counter address; both modern
hydration hooks now have a heap-debug regression that also executes on ARM64.

The separate nonempty modern hydration-data owner issue is tracked as Code Journal
task `a1321652`; its parse/insert path is unchanged from main. The focused nested
reference regression uses explicit empty modern payloads to isolate reference state.


## CI follow-up after the serialization review

- [x] Scope whole-module assertions to their fixture functions after Core prelude injection.
- [x] Audit aligned dynamic serialization snapshots without enlarging the ABI allowlist.
- [x] Validate and reduce repeated work in the archived REPL cache regression.
- [x] Schedule each target of the argument-owner probe independently under the existing timeout.
- [ ] Publish focused validation and verify CI and Greptile on the final commit.

Run `39038329288` repeated three structural assertion failures and one ABI-audit
limitation across executable platforms. Linux ARM64 also reached the global
60-second limit in a five-target argument-owner test, and the macOS archived
REPL probe reached 240 seconds while rerunning the broader cache lifecycle test.

Focused validation passes 40 library/CLI cases under the CI profile, including
all five targets, the complete x86 runtime alignment audit, and two negative
audit regressions. The archived REPL cache test passes in 86.48 seconds. The
compiler build is warning-free, Rustdoc checks and `git diff --check` pass, and
no global test timeout or production runtime behavior changes in this follow-up.


## Independent Grok review correction

- [x] Reproduce default ErrorException construction lines on the reviewed head.
- [x] Stamp ordinary Throwable construction lines after defaults for fixed, dynamic, and late-static allocation.
- [x] Verify native inheritance, factories, delayed throws, explicit zero/null overrides, and native-to-eval getters.
- [x] Check all three allocation paths on all five targets and assemble representative programs.

Grok delegation `14b44edf-751e-438e-9b98-064b8b1b031f` found one actionable
defect on `c5cbece4b4`: a default ErrorException line read zero because ordinary
property initialization cleared the inherited creation-line word. Five focused
codegen tests and the all-target allocation test validate the repair. Review
outcomes for the final published head remain tracked in Code Journal and on PR1870.

The broader opaque-eval construction-location gap affects existing Exception as
well and is task `5b678980`. Separate owner-retention controls using compact
Exception and stdClass are tasks `f53ab0fe` and `6436dcd1`. The new native and
bridge regressions each retain their clean-heap assertions.


## Second independent Grok correction

- [x] Reproduce protected `code` assignment clearing construction metadata.
- [x] Reserve ordinary Throwable metadata after PHP properties and the optional dynamic hash.
- [x] Keep property/GC payload sizes distinct from full allocation sizes.
- [x] Update allocation, location readers, uncaught reporting, and shallow-copy storage on both architectures.
- [x] Add scalar, compound, dynamic-name and eval Reflection write regressions.

Grok delegation `74583fbf-67ea-42f2-86fc-b1000645293d` found that the first
repair still reused the high word of `code`. The second repair gives ordinary
Throwables a separate metadata word. Dynamic-name scalar writes have an existing
boxing-owner leak even for a plain user class (Code Journal task `0a0ba2ab`), and
Reflection metadata retention remains task `7215b48c`. Those functional probes
check line preservation; the native scalar/compound and native-to-eval location
probes separately require a clean heap.
