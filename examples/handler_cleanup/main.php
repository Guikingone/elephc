<?php
// Handler captures stay alive through the program and are destroyed at shutdown.
class HandlerOwner {
    public function __construct(public string $name) {}
    public function __destruct() {echo "released ", $this->name, "\n";}
}
$errorOwner = new HandlerOwner('error handler');
set_error_handler(function($level, $message) use ($errorOwner) {return false;});
unset($errorOwner);
$exceptionOwner = new HandlerOwner('exception handler');
set_exception_handler(function($exception) use ($exceptionOwner) {});
unset($exceptionOwner);
echo "program body\n";
