<?php

class Account
{
    public int $id = 1;
    private string $token = 'initial';

    public function setToken(string $token): void
    {
        $this->token = $token;
    }

    public function __debugInfo(): array
    {
        return ['id' => $this->id];
    }
}

$first = new Account();
$second = new Account();
var_dump($first == $second);
$second->setToken('changed');
var_dump($first == $second);
