//! Purpose:
//! The `mysqli_result` class (plus its foreach iterator), as an elephc-PHP
//! source fragment. A result OWNS its rows: `mysqli::query` drains every row
//! through `elephc_pdo_step` into PHP arrays and finalizes the statement before
//! the result is handed out, so a later query on the same connection can never
//! invalidate an earlier result (`data_seek`, `num_rows`, `foreach` all keep
//! working).
//!
//! Called from:
//! - `crate::mysqli_prelude::fragments::source_for_version` (concatenated into
//!   the injected prelude).
//!
//! Key details:
//! - Rows are stored positionally; assoc/both shapes are derived from the
//!   captured column-name list at fetch time (duplicate names: last wins, like
//!   PHP). Field metadata (`name`, `table`, native type, flags, length) is
//!   captured at drain time because the statement is finalized immediately.
//! - `fetch_*` return `null` when the cursor is exhausted (`fetch_column`
//!   returns `false`), matching php-src.
//! - `fetch_column` is PHP 8.1+; its block is version-gated in `fragments.rs`.
//! - `getIterator()` yields `fetch_assoc()` rows from position 0 (PHP's
//!   `mysqli_result` foreach is assoc with integer keys).

/// The `mysqli_result` class and its iterator (fragment without a `<?php`
/// header).
pub(super) const SRC: &str = r#"
// -- mysqli buffered result surface --

class mysqli_result implements IteratorAggregate {
    public int $num_rows = 0;
    public int $field_count = 0;
    // Column byte lengths of the row returned by the most recent fetch, or
    // null before the first fetch / after exhaustion (php-src's $lengths).
    public ?array $lengths = null;

    // Buffered cells and drain-time column metadata. Cells are stored FLAT in
    // row-major order (row * field_count + column) so every buffered value is
    // a Mixed scalar — a fetch builds a fresh row array, which keeps the
    // checker's array element typing simple (no nested-array Mixed unwraps).
    // The bridge statement is already finalized when a result exists, so
    // everything a fetch needs lives here.
    private array $cells = [];
    private array $names = [];
    private array $metaTables = [];
    private array $metaNativeTypes = [];
    private array $metaFlags = [];
    private array $metaLens = [];
    private array $metaDecimals = [];
    private int $rowCount = 0;
    private int $pos = 0;
    private int $fieldPos = 0;

    // Internal factory used by mysqli::query's drain (not part of PHP's
    // surface; a user never constructs mysqli_result directly). `$cells` is
    // the flat row-major cell list described above.
    private static function __elephcFromDrain(array $cells, int $rowCount, array $names, array $tables, array $nativeTypes, array $flags, array $lens, array $decimals): mysqli_result {
        $_result = new mysqli_result();
        $_result->cells = $cells;
        $_result->rowCount = $rowCount;
        $_result->names = $names;
        $_result->metaTables = $tables;
        $_result->metaNativeTypes = $nativeTypes;
        $_result->metaFlags = $flags;
        $_result->metaLens = $lens;
        $_result->metaDecimals = $decimals;
        $_result->num_rows = $rowCount;
        $_result->field_count = count($names);
        return $_result;
    }

    public function fetch_row(): ?array {
        if ($this->pos >= $this->rowCount) {
            $this->lengths = null;
            return null;
        }
        $_base = $this->pos * $this->field_count;
        $this->pos = $this->pos + 1;
        $_row = [];
        $_lengths = [];
        for ($_i = 0; $_i < $this->field_count; $_i++) {
            $_value = $this->cells[$_base + $_i];
            $_row[] = $_value;
            if ($_value === null) {
                $_lengths[] = 0;
            } else {
                $_lengths[] = strlen((string) $_value);
            }
        }
        $this->lengths = $_lengths;
        return $_row;
    }

    public function fetch_assoc(): ?array {
        $_row = $this->fetch_row();
        if ($_row === null) {
            return null;
        }
        $_assoc = [];
        for ($_i = 0; $_i < $this->field_count; $_i++) {
            // Duplicate column names: last wins, matching PHP's assoc shape.
            $_assoc[(string) $this->names[$_i]] = $_row[$_i];
        }
        return $_assoc;
    }

    public function fetch_array(int $mode = 3): ?array {
        if ($mode != 1 && $mode != 2 && $mode != 3) {
            throw new ValueError("mysqli_result::fetch_array(): Argument #1 (\$mode) must be one of MYSQLI_NUM, MYSQLI_ASSOC, or MYSQLI_BOTH");
        }
        $_row = $this->fetch_row();
        if ($_row === null) {
            return null;
        }
        if ($mode == 2) {
            return $_row;
        }
        $_out = [];
        if ($mode == 3) {
            for ($_i = 0; $_i < $this->field_count; $_i++) {
                $_out[$_i] = $_row[$_i];
            }
        }
        for ($_i = 0; $_i < $this->field_count; $_i++) {
            $_out[(string) $this->names[$_i]] = $_row[$_i];
        }
        return $_out;
    }

    public function fetch_object(string $class = "stdClass", array $constructor_args = []): mixed {
        $_row = $this->fetch_row();
        if ($_row === null) {
            return null;
        }
        // Two straight-line returns rather than one reassigned local, same
        // reason as the PDO prelude's hydrateClassOrStd: the dynamic-new result
        // is Mixed, and rebinding a concrete `new stdClass()` into that local
        // would unify Mixed with Object(stdClass) in a slot that is about to be
        // dynamic-property-written. stdClass also never goes through dynamic
        // allocation (php-src resolves it to zend_standard_class_def directly).
        if (strtolower($class) === "stdclass") {
            $_std = new stdClass();
            for ($_i = 0; $_i < $this->field_count; $_i++) {
                $_n = (string) $this->names[$_i];
                $_std->{$_n} = $_row[$_i];
            }
            return $_std;
        }
        // Documented divergence: php-src assigns properties BEFORE calling the
        // constructor; elephc constructs first (PDO::FETCH_PROPS_LATE order) so
        // the dynamic instantiation goes through the one proven `new $class`
        // path with constructor arguments.
        $_object = new $class(...$constructor_args);
        for ($_i = 0; $_i < $this->field_count; $_i++) {
            $_n = (string) $this->names[$_i];
            $_object->{$_n} = $_row[$_i];
        }
        return $_object;
    }

    public function fetch_all(int $mode = 2): array {
        if ($mode != 1 && $mode != 2 && $mode != 3) {
            throw new ValueError("mysqli_result::fetch_all(): Argument #1 (\$mode) must be one of MYSQLI_NUM, MYSQLI_ASSOC, or MYSQLI_BOTH");
        }
        $_all = [];
        // A fresh `$_row` each iteration, narrowed by the `=== null` break, so it
        // is never reassigned from the narrowed `array` back to `?array` (which
        // the checker rejects inside the narrowed scope).
        while (true) {
            $_row = $this->fetch_array($mode);
            if ($_row === null) {
                break;
            }
            $_all[] = $_row;
        }
        return $_all;
    }

    // -- elephc PHP >= 8.1 mysqli fetch_column begin --
    public function fetch_column(int $column = 0): mixed {
        if ($column < 0 || $column >= $this->field_count) {
            throw new ValueError("mysqli_result::fetch_column(): Argument #1 (\$column) must be greater than or equal to 0 and less than the number of fields in this result set");
        }
        $_row = $this->fetch_row();
        if ($_row === null) {
            return false;
        }
        return $_row[$column];
    }
    // -- elephc PHP >= 8.1 mysqli fetch_column end --

    public function data_seek(int $offset): bool {
        if ($offset < 0 || $offset >= $this->rowCount) {
            return false;
        }
        $this->pos = $offset;
        return true;
    }

    public function fetch_field_direct(int $index): mixed {
        if ($index < 0 || $index >= $this->field_count) {
            return false;
        }
        // Unknown metadata the bridge does not expose stays 0 / "" (documented);
        // name/orgname, table/orgtable, type, flags, and length are real.
        $_field = new stdClass();
        $_field->{"name"} = (string) $this->names[$index];
        $_field->{"orgname"} = (string) $this->names[$index];
        $_field->{"table"} = (string) $this->metaTables[$index];
        $_field->{"orgtable"} = (string) $this->metaTables[$index];
        $_field->{"def"} = "";
        $_field->{"db"} = "";
        $_field->{"catalog"} = "def";
        $_field->{"max_length"} = 0;
        $_field->{"length"} = (int) $this->metaLens[$index];
        $_field->{"charsetnr"} = 0;
        $_field->{"flags"} = (int) $this->metaFlags[$index];
        $_field->{"type"} = $this->mysqliTypeFromNative((string) $this->metaNativeTypes[$index]);
        // The bridge's column_precision is MySQL's own wire "decimals" byte.
        $_field->{"decimals"} = (int) $this->metaDecimals[$index];
        return $_field;
    }

    public function fetch_field(): mixed {
        if ($this->fieldPos >= $this->field_count) {
            return false;
        }
        $_field = $this->fetch_field_direct($this->fieldPos);
        $this->fieldPos = $this->fieldPos + 1;
        return $_field;
    }

    public function fetch_fields(): array {
        $_fields = [];
        for ($_i = 0; $_i < $this->field_count; $_i++) {
            $_fields[] = $this->fetch_field_direct($_i);
        }
        return $_fields;
    }

    public function field_seek(int $index): bool {
        // Moves the fetch_field() cursor. php returns the previous position;
        // elephc returns true/false (in-range), the mysqli_result::field_seek
        // contract most code relies on.
        if ($index < 0 || $index > $this->field_count) {
            return false;
        }
        $this->fieldPos = $index;
        return true;
    }

    public function field_tell(): int {
        return $this->fieldPos;
    }

    public function close(): void {
        // Drop the buffered cells; further fetches see an exhausted cursor.
        $this->cells = [];
        $this->rowCount = 0;
        $this->num_rows = 0;
        $this->pos = 0;
        $this->lengths = null;
    }

    public function free(): void {
        $this->close();
    }

    public function free_result(): void {
        $this->close();
    }

    public function getIterator(): Iterator {
        return new __ElephcMysqliResultIterator($this);
    }

    // php-src type_to_name_native reversed: the bridge reports MySQL's own
    // wire-type names; map them onto the MYSQLI_TYPE_* integers with
    // MYSQLI_TYPE_STRING as the default for anything unrecognized.
    private function mysqliTypeFromNative(string $native): int {
        if ($native === "DECIMAL") { return 0; }
        if ($native === "TINY") { return 1; }
        if ($native === "SHORT") { return 2; }
        if ($native === "LONG") { return 3; }
        if ($native === "FLOAT") { return 4; }
        if ($native === "DOUBLE") { return 5; }
        if ($native === "NULL") { return 6; }
        if ($native === "TIMESTAMP") { return 7; }
        if ($native === "LONGLONG") { return 8; }
        if ($native === "INT24") { return 9; }
        if ($native === "DATE") { return 10; }
        if ($native === "TIME") { return 11; }
        if ($native === "DATETIME") { return 12; }
        if ($native === "YEAR") { return 13; }
        if ($native === "NEWDATE") { return 14; }
        if ($native === "VARCHAR") { return 15; }
        if ($native === "BIT") { return 16; }
        if ($native === "JSON") { return 245; }
        if ($native === "NEWDECIMAL") { return 246; }
        if ($native === "ENUM") { return 247; }
        if ($native === "SET") { return 248; }
        if ($native === "TINY_BLOB") { return 249; }
        if ($native === "MEDIUM_BLOB") { return 250; }
        if ($native === "LONG_BLOB") { return 251; }
        if ($native === "BLOB") { return 252; }
        if ($native === "VAR_STRING") { return 253; }
        if ($native === "STRING") { return 254; }
        if ($native === "GEOMETRY") { return 255; }
        return 254;
    }
}

// Iterator behind mysqli_result::getIterator(): assoc rows keyed by position,
// starting from row 0 regardless of the fetch cursor (rewind() seeks).
final class __ElephcMysqliResultIterator implements Iterator {
    private mysqli_result $result;
    private mixed $row;
    private int $position;

    public function __construct(mysqli_result $result) {
        $this->result = $result;
        $this->row = null;
        $this->position = 0;
    }

    public function rewind(): void {
        $this->result->data_seek(0);
        $this->row = $this->result->fetch_assoc();
        $this->position = 0;
    }

    public function valid(): bool {
        return $this->row !== null;
    }

    public function current(): mixed {
        return $this->row;
    }

    public function key(): mixed {
        return $this->position;
    }

    public function next(): void {
        $this->row = $this->result->fetch_assoc();
        $this->position = $this->position + 1;
    }
}
"#;
