//! Purpose:
//! Checks owners retained by user stream-wrapper and filter registrations.
//!
//! Called from:
//! - The executable runtime GC suite and the dedicated fileinfo CI gates.
//!
//! Key details:
//! - Unregistration retires metadata while already-open stream objects stay usable.
//! - Registrations without an explicit removal API retire at normal program shutdown.

use crate::support::*;

/// Registration strings and opened wrapper objects retire on normal and exceptional close paths.
#[test]
fn test_fileinfo_stream_registration_owners() {
    let (out, _) = compile_and_run_with_heap_debug_and_asm(r#"<?php
class FileinfoRegistryWrapper {
    public $context;
    public static int $dropped = 0;
    public static int $closed = 0;
    private int $position = 0;
    private string $payload = "";
    public function stream_open(string $path, string $mode, int $options, ?string &$opened): bool { $this->payload = str_repeat("x", 41); return true; }
    public function stream_stat(): array { return ["mode" => 0100644, "size" => 5]; }
    public function stream_read(int $count): string { $s = substr("alive", $this->position, $count); $this->position += strlen($s); return $s; }
    public function stream_eof(): bool { return $this->position >= 5; }
    public function __destruct() { FileinfoRegistryWrapper::$dropped++; }
}
class FileinfoRegistryClosing extends FileinfoRegistryWrapper {
    public function stream_close(): void { FileinfoRegistryWrapper::$closed++; }
}
class FileinfoRegistryThrowing extends FileinfoRegistryWrapper {
    public function stream_close(): void { FileinfoRegistryWrapper::$closed++; throw new Exception("close"); }
}
class FileinfoRegistryFilter extends php_user_filter {}
for ($i = 0; $i < 40; $i++) {
    $name = "fileinfolease" . $i;
    if (!stream_wrapper_register($name, "FileinfoRegistryWrapper")) { echo "register failed\n"; }
    if (!stream_wrapper_unregister($name)) { echo "unregister failed\n"; }
    if (@stream_wrapper_unregister($name)) { echo "duplicate unregister\n"; }
}
stream_wrapper_register("fileinfoheld", "FileinfoRegistryWrapper");
$stream = fopen("fileinfoheld://x", "rb");
stream_wrapper_unregister("fileinfoheld");
echo stream_get_contents($stream), "\n";
fclose($stream);
stream_wrapper_register("fileinfoclosing", "FileinfoRegistryClosing");
stream_wrapper_register("fileinfothrowing", "FileinfoRegistryThrowing");
for ($i = 0; $i < 8; $i++) {
    $stream = fopen("fileinfoclosing://x", "rb"); fclose($stream);
    $stream = fopen("fileinfothrowing://x", "rb");
    try { fclose($stream); } catch (Exception $e) { if ($e->getMessage() !== "close") { echo "wrong exception\n"; } }
}
echo FileinfoRegistryWrapper::$dropped, ":", FileinfoRegistryWrapper::$closed, "\n";
stream_wrapper_register("fileinfofinal", "FileinfoRegistryWrapper");
stream_filter_register("fileinfo.persist", "FileinfoRegistryFilter");
unset($name, $stream, $e);
echo "done\n";
"#);
    assert!(out.success, "{}", out.stderr);
    assert_eq!(out.stdout, "alive\n17:16\ndone\n", "{}", out.stderr);
    assert!(out.stderr.contains("HEAP DEBUG: leak summary: clean"), "{}", out.stderr);
}
